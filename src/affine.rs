use ark_ec::{AffineRepr, CurveConfig, CurveGroup, PrimeGroup};
use ark_serialize::{
    CanonicalDeserialize, CanonicalSerialize, Compress, SerializationError, Valid, Validate,
};
use ark_std::{
    borrow::Borrow,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    io::{Read, Write},
    ops::{Add, Mul, Neg, Sub},
    rand::{
        Rng,
        distributions::{Distribution, Standard},
    },
    vec::*,
};
use std::hash::{Hash, Hasher};
use std::io::ErrorKind;
use std::os::raw::c_void;
use std::{fmt, io};

use ark_ff::{PrimeField, ToConstraintField, fields::Field};

use crate::bigint_to_le_bytes;
use crate::group::Xsk233Projective;
use crate::xsk233::Xsk233CurveConfig;
use educe::Educe;
use xs233_sys::{
    xsk233_encode, xsk233_equals, xsk233_generator, xsk233_mul_frob, xsk233_neg, xsk233_neutral,
    xsk233_point,
};
use zeroize::Zeroize;

const COMPRESSED_POINT_SIZE: usize = 30;

/// from xsk233_equals : -1 if two points are equal and 0 if not. This is -1.
pub(crate) const C_XSK233_EQUALS_TRUE: u32 = 0xFFFFFFFFu32;

/// Affine coordinates for a point on an elliptic curve in short Weierstrass
/// form, over the base field `P::BaseField`.
#[derive(Educe)]
#[educe(Copy, Clone)]
#[must_use]
pub struct Xsk233Affine(xsk233_point);

impl Xsk233Affine {
    pub fn new_unchecked(point: xsk233_point) -> Self {
        Self(point)
    }

    pub fn inner(&self) -> &xsk233_point {
        &self.0
    }

    pub fn into_inner(self) -> xsk233_point {
        self.0
    }
}

impl Eq for Xsk233Affine {}

impl PartialEq<Self> for Xsk233Affine {
    fn eq(&self, other: &Self) -> bool {
        self.into_group() == other.into_group()
    }
}

impl PartialEq<Xsk233Projective> for Xsk233Affine {
    fn eq(&self, other: &Xsk233Projective) -> bool {
        unsafe { C_XSK233_EQUALS_TRUE == xsk233_equals(self.inner(), other.inner()) }
    }
}

impl Hash for Xsk233Affine {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&self.into_group(), state);
    }
}

impl Display for Xsk233Affine {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut ser = Vec::new();
        self.serialize_compressed(&mut ser)
            .map_err(|_| fmt::Error)?;

        write!(f, "{}", hex::encode(ser))
    }
}

impl Debug for Xsk233Affine {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut ser = Vec::new();
        self.serialize_compressed(&mut ser)
            .map_err(|_| fmt::Error)?;

        write!(f, "{}", hex::encode(ser))
    }
}

impl Zeroize for Xsk233Affine {
    // The phantom data does not contain element-specific data
    // and thus does not need to be zeroized.
    fn zeroize(&mut self) {
        unimplemented!("xsk233-sys crate does not implement zeroize")
    }
}

impl Distribution<Xsk233Affine> for Standard {
    /// Generates a uniformly random instance of the curve.
    #[inline]
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Xsk233Affine {
        let rand: Xsk233Projective = rng.r#gen();
        rand.into()
    }
}

impl AffineRepr for Xsk233Affine {
    type Config = Xsk233CurveConfig;
    type BaseField = <Xsk233CurveConfig as CurveConfig>::BaseField;
    type ScalarField = <Xsk233CurveConfig as CurveConfig>::ScalarField;
    type Group = Xsk233Projective;

    fn xy(&self) -> Option<(Self::BaseField, Self::BaseField)> {
        unimplemented!(
            "xsk233-sys crate that is used under the hood does not
        allow to operate with x and y concepts. Therefore, there is no direct way in getting
        x and y coordinates."
        )
    }

    fn is_zero(&self) -> bool {
        unsafe { C_XSK233_EQUALS_TRUE == xsk233_equals(&xsk233_neutral, &self.0) }
    }

    #[inline]
    fn generator() -> Self {
        unsafe { Xsk233Affine(xsk233_generator) }
    }

    fn zero() -> Self {
        unsafe { Self(xsk233_neutral) }
    }

    fn from_random_bytes(bytes: &[u8]) -> Option<Self> {
        if let Ok(p) = Xsk233Projective::deserialize_compressed(bytes) {
            return Some(p.into_affine());
        }

        None
    }

    fn mul_bigint(&self, by: impl AsRef<[u64]>) -> Self::Group {
        self.into_group().mul_bigint(by)
    }

    /// Multiplies this element by the cofactor and output the
    /// resulting projective element.
    fn mul_by_cofactor_to_group(&self) -> Self::Group {
        self.mul(Self::ScalarField::from(
            *Self::Config::COFACTOR.first().unwrap(),
        ))
    }

    /// Performs cofactor clearing.
    /// The default method is simply to multiply by the cofactor.
    /// Some curves can implement a more efficient algorithm.
    fn clear_cofactor(&self) -> Self {
        self.mul_by_cofactor_to_group().into_affine()
    }
}

impl Neg for Xsk233Affine {
    type Output = Self;

    /// If `self.is_zero()`, returns `self` (`== Self::zero()`).
    /// Else, returns `(x, -y)`, where `self = (x, y)`.
    #[inline]
    fn neg(mut self) -> Self {
        unsafe {
            xsk233_neg(&mut self.0, &self.0);
            self
        }
    }
}

impl<T: Borrow<Self>> Add<T> for Xsk233Affine {
    type Output = Xsk233Projective;
    fn add(self, other: T) -> Xsk233Projective {
        let mut copy = self.into_group();
        copy += other.borrow();
        copy
    }
}

impl Add<Xsk233Projective> for Xsk233Affine {
    type Output = Xsk233Projective;
    fn add(self, other: Xsk233Projective) -> Xsk233Projective {
        other + self
    }
}

impl<'a> Add<&'a Xsk233Projective> for Xsk233Affine {
    type Output = Xsk233Projective;
    fn add(self, other: &'a Xsk233Projective) -> Xsk233Projective {
        *other + self
    }
}

impl<T: Borrow<Self>> Sub<T> for Xsk233Affine {
    type Output = Xsk233Projective;
    fn sub(self, other: T) -> Xsk233Projective {
        let mut copy = self.into_group();
        copy -= other.borrow();
        copy
    }
}

impl Sub<Xsk233Projective> for Xsk233Affine {
    type Output = Xsk233Projective;
    fn sub(self, other: Xsk233Projective) -> Xsk233Projective {
        self + (-other)
    }
}

impl<'a> Sub<&'a Xsk233Projective> for Xsk233Affine {
    type Output = Xsk233Projective;
    fn sub(self, other: &'a Xsk233Projective) -> Xsk233Projective {
        self + (-*other)
    }
}

impl Default for Xsk233Affine {
    #[inline]
    fn default() -> Self {
        unsafe { Xsk233Affine(xsk233_neutral) }
    }
}

impl<T: Borrow<<Xsk233CurveConfig as CurveConfig>::ScalarField>> Mul<T> for Xsk233Affine {
    type Output = Xsk233Projective;

    #[inline]
    fn mul(self, other: T) -> Self::Output {
        unsafe {
            let scalar_bytes = bigint_to_le_bytes(other.borrow().into_bigint());
            let mut result = xsk233_neutral;
            xsk233_mul_frob(
                &mut result,
                &self.0,
                scalar_bytes.as_ptr() as *const _,
                scalar_bytes.len(),
            );

            Self::Output::new_unchecked(result)
        }
    }
}

impl From<Xsk233Projective> for Xsk233Affine {
    #[inline]
    fn from(p: Xsk233Projective) -> Xsk233Affine {
        Self(p.into_inner())
    }
}

impl CanonicalSerialize for Xsk233Affine {
    #[inline]
    fn serialize_with_mode<W: Write>(
        &self,
        mut writer: W,
        compress: Compress,
    ) -> Result<(), SerializationError> {
        if compress == Compress::No {
            return Err(SerializationError::IoError(io::Error::new(
                ErrorKind::Unsupported,
                "serialization without compression is not supported",
            )));
        }

        unsafe {
            let pt = self.0;
            let mut dst = [0u8; 30];
            xsk233_encode(dst.as_mut_ptr() as *mut c_void, &pt);

            writer.write_all(dst.as_mut_slice())?;
        }

        Ok(())
    }

    #[inline]
    fn serialized_size(&self, _compress: Compress) -> usize {
        COMPRESSED_POINT_SIZE
    }
}

impl Valid for Xsk233Affine {
    fn check(&self) -> Result<(), SerializationError> {
        // it is assumed that all points are created from :
        // a. decode function
        // b. multipling by a scalar a point like generator point.
        //
        // Option b is valid by nature.
        // Option a has a really intricate mechanism of rejecting invalid points.
        Ok(())
    }
}

impl CanonicalDeserialize for Xsk233Affine {
    fn deserialize_with_mode<R: Read>(
        reader: R,
        compress: Compress,
        validate: Validate,
    ) -> Result<Self, SerializationError> {
        Xsk233Projective::deserialize_with_mode(reader, compress, validate).map(|p| p.into_affine())
    }
}

impl<ConstraintF: Field> ToConstraintField<ConstraintF> for Xsk233Affine {
    #[inline]
    fn to_field_elements(&self) -> Option<Vec<ConstraintF>> {
        unimplemented!(
            "xsk233-sys crate that is used under the hood does not
        allow to operate with x and y concepts. Therefore, there is no direct way in getting
        field elements."
        )
    }
}
