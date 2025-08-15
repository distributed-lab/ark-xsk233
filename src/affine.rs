use ark_ec::{AffineRepr, CurveConfig};
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
use std::{fmt, io};
use std::io::ErrorKind;
use std::os::raw::c_void;

use ark_ff::{PrimeField, ToConstraintField, fields::Field};

use crate::bigint_to_le_bytes;
use crate::group::Xsk233Projective;
use crate::xsk233::Xsk233CurveConfig;
use educe::Educe;
use xs233_sys::{
    xsk233_decode, xsk233_encode, xsk233_generator, xsk233_mul_frob, xsk233_neg, xsk233_neutral,
    xsk233_point,
};
use zeroize::Zeroize;

const COMPRESSED_POINT_SIZE: usize = 30;

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
    fn eq(&self, _other: &Self) -> bool {
        unimplemented!()
    }
}

impl PartialEq<Xsk233Projective> for Xsk233Affine {
    fn eq(&self, _other: &Xsk233Projective) -> bool {
        unimplemented!()
    }
}

impl Hash for Xsk233Affine {
    fn hash<H: Hasher>(&self, _state: &mut H) {
        unimplemented!()
    }
}

impl Display for Xsk233Affine {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut ser = Vec::new();
        self.serialize_compressed(&mut ser).map_err(|_| fmt::Error)?;

        write!(f, "{}", hex::encode(ser))
    }
}

impl Debug for Xsk233Affine {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut ser = Vec::new();
        self.serialize_compressed(&mut ser).map_err(|_| fmt::Error)?;

        write!(f, "{}", hex::encode(ser))
    }
}

impl Zeroize for Xsk233Affine {
    // The phantom data does not contain element-specific data
    // and thus does not need to be zeroized.
    fn zeroize(&mut self) {
        unimplemented!()
    }
}

impl Distribution<Xsk233Affine> for Standard {
    /// Generates a uniformly random instance of the curve.
    #[inline]
    fn sample<R: Rng + ?Sized>(&self, _rng: &mut R) -> Xsk233Affine {
        unimplemented!()
    }
}

impl AffineRepr for Xsk233Affine {
    type Config = Xsk233CurveConfig;
    type BaseField = <Xsk233CurveConfig as CurveConfig>::BaseField;
    type ScalarField = <Xsk233CurveConfig as CurveConfig>::ScalarField;
    type Group = Xsk233Projective;

    fn xy(&self) -> Option<(Self::BaseField, Self::BaseField)> {
        unimplemented!()
    }

    #[inline]
    fn generator() -> Self {
        unsafe { Xsk233Affine(xsk233_generator) }
    }

    fn zero() -> Self {
        unimplemented!()
    }

    fn from_random_bytes(bytes: &[u8]) -> Option<Self> {
        unsafe {
            let mut result = xsk233_neutral;
            let success = xsk233_decode(&mut result, bytes.as_ptr() as *mut c_void);
            if success != 0 {
                return Some(Xsk233Affine(result));
            }

            None
        }
    }

    fn mul_bigint(&self, _by: impl AsRef<[u64]>) -> Self::Group {
        unimplemented!()
    }

    /// Multiplies this element by the cofactor and output the
    /// resulting projective element.
    fn mul_by_cofactor_to_group(&self) -> Self::Group {
        unimplemented!()
    }

    /// Performs cofactor clearing.
    /// The default method is simply to multiply by the cofactor.
    /// Some curves can implement a more efficient algorithm.
    fn clear_cofactor(&self) -> Self {
        unimplemented!()
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
        unimplemented!()
    }
}

impl CanonicalDeserialize for Xsk233Affine {
    fn deserialize_with_mode<R: Read>(
        _reader: R,
        _compress: Compress,
        _validate: Validate,
    ) -> Result<Self, SerializationError> {
        unimplemented!()
    }
}

impl<ConstraintF: Field> ToConstraintField<ConstraintF> for Xsk233Affine {
    #[inline]
    fn to_field_elements(&self) -> Option<Vec<ConstraintF>> {
        unimplemented!()
    }
}
