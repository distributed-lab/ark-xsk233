use crate::affine::Xsk233Affine;
use crate::xsk233::{Fr, Xsk233CurveConfig};
use crate::{bigint_to_le_bytes, impl_additive_ops_from_ref};
use ark_ec::short_weierstrass::SWCurveConfig;
use ark_ec::{AffineRepr, CurveConfig, CurveGroup, PrimeGroup, ScalarMul, VariableBaseMSM};
use ark_ff::{AdditiveGroup, PrimeField, ToConstraintField, fields::Field};
use ark_serialize::{
    CanonicalDeserialize, CanonicalSerialize, Compress, SerializationError, Valid, Validate,
};
use ark_std::{
    UniformRand, Zero,
    borrow::Borrow,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    hash::{Hash, Hasher},
    io::{Read, Write},
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
    rand::{
        Rng,
        distributions::{Distribution, Standard},
    },
    vec::*,
};
use educe::Educe;
use std::io;
use std::io::ErrorKind;
use std::os::raw::c_void;
use xs233_sys::{
    xsk233_add, xsk233_decode, xsk233_double, xsk233_encode, xsk233_equals, xsk233_mul_frob,
    xsk233_neg, xsk233_neutral, xsk233_point, xsk233_sub,
};
use zeroize::Zeroize;

#[derive(Educe)]
#[educe(Copy, Clone)]
#[must_use]
pub struct Xsk233Projective(xsk233_point);

impl Xsk233Projective {
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

impl Display for Xsk233Projective {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", Xsk233Affine::from(*self))
    }
}

impl Debug for Xsk233Projective {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", Xsk233Affine::from(*self))
    }
}

impl Eq for Xsk233Projective {}
impl PartialEq for Xsk233Projective {
    fn eq(&self, other: &Self) -> bool {
        unsafe { xsk233_equals(self.inner(), other.inner()) != 0 }
    }
}

impl PartialEq<Xsk233Affine> for Xsk233Projective {
    fn eq(&self, other: &Xsk233Affine) -> bool {
        unsafe { xsk233_equals(self.inner(), other.inner()) != 0 }
    }
}

impl Hash for Xsk233Projective {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut ser = Vec::new();
        if let Ok(()) = self.serialize_compressed(&mut ser) {
            Hash::hash(&ser, state);
        }
    }
}

impl Zeroize for Xsk233Projective {
    fn zeroize(&mut self) {
        unimplemented!("xsk233-sys does not implement zeroize")
    }
}

impl Distribution<Xsk233Projective> for Standard {
    /// Generates a uniformly random instance of the curve.
    #[inline]
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Xsk233Projective {
        Xsk233Projective::generator() * Fr::rand(rng)
    }
}

impl Default for Xsk233Projective {
    #[inline]
    fn default() -> Self {
        Xsk233Projective::zero()
    }
}

impl Zero for Xsk233Projective {
    /// Returns the point at infinity, which always has Z = 0.
    #[inline]
    fn zero() -> Self {
        unsafe { Self::new_unchecked(xsk233_neutral) }
    }

    /// Checks whether `self.z.is_zero()`.
    #[inline]
    fn is_zero(&self) -> bool {
        self == &Self::zero()
    }
}

impl Xsk233Projective {
    const fn zero() -> Xsk233Projective {
        unsafe { Self(xsk233_neutral) }
    }
}

impl AdditiveGroup for Xsk233Projective {
    type Scalar = <Xsk233CurveConfig as CurveConfig>::ScalarField;

    const ZERO: Self = Xsk233Projective::zero();

    fn double_in_place(&mut self) -> &mut Self {
        unsafe {
            xsk233_double(&mut self.0, &self.0);
            self
        }
    }
}

impl PrimeGroup for Xsk233Projective {
    type ScalarField = <Xsk233CurveConfig as CurveConfig>::ScalarField;

    #[inline]
    fn generator() -> Self {
        Xsk233Affine::generator().into()
    }

    #[inline]
    fn mul_bigint(&self, other: impl AsRef<[u64]>) -> Self {
        let words = other.as_ref();

        // Convert to bytes in little-endian order
        let mut bytes = Vec::with_capacity(words.len() * 8);
        for word in words {
            bytes.extend_from_slice(&word.to_le_bytes());
        }

        let scalar = Self::ScalarField::from_le_bytes_mod_order(&bytes);
        self.mul(scalar)
    }
}

impl CurveGroup for Xsk233Projective {
    type Config = Xsk233CurveConfig;
    type BaseField = <Xsk233CurveConfig as CurveConfig>::BaseField;
    type Affine = Xsk233Affine;
    type FullGroup = Xsk233Affine;

    /// Normalizes a slice of projective elements so that
    /// conversion to affine is cheap.
    #[inline]
    fn normalize_batch(_v: &[Self]) -> Vec<Self::Affine> {
        unimplemented!(
            "xsk233_point structure is used in both affine
        and projective coordinates so there is no sense in normalization."
        )
    }
}

impl Neg for Xsk233Projective {
    type Output = Self;

    #[inline]
    fn neg(mut self) -> Self {
        unsafe {
            xsk233_neg(&mut self.0, &self.0);
            self
        }
    }
}

impl<T: Borrow<Xsk233Affine>> AddAssign<T> for Xsk233Projective {
    fn add_assign(&mut self, other: T) {
        unsafe {
            xsk233_add(&mut self.0, &self.0, other.borrow().inner());
        }
    }
}

impl<T: Borrow<Xsk233Affine>> Add<T> for Xsk233Projective {
    type Output = Self;
    fn add(mut self, other: T) -> Self {
        let other = other.borrow();
        self += other;
        self
    }
}

impl<T: Borrow<Xsk233Affine>> SubAssign<T> for Xsk233Projective {
    fn sub_assign(&mut self, other: T) {
        unsafe {
            xsk233_sub(&mut self.0, &self.0, other.borrow().inner());
        }
    }
}

impl<T: Borrow<Xsk233Affine>> Sub<T> for Xsk233Projective {
    type Output = Self;
    fn sub(mut self, other: T) -> Self {
        self -= other.borrow();
        self
    }
}

impl_additive_ops_from_ref!(Xsk233Projective);

impl<'a> Add<&'a Self> for Xsk233Projective {
    type Output = Self;

    #[inline]
    fn add(mut self, other: &'a Self) -> Self {
        self += other;
        self
    }
}

impl<'a> AddAssign<&'a Self> for Xsk233Projective {
    fn add_assign(&mut self, other: &'a Self) {
        unsafe {
            xsk233_add(&mut self.0, &self.0, other.inner());
        }
    }
}

impl<'a> Sub<&'a Self> for Xsk233Projective {
    type Output = Self;

    #[inline]
    fn sub(mut self, other: &'a Self) -> Self {
        self -= other;
        self
    }
}

impl<'a> SubAssign<&'a Self> for Xsk233Projective {
    fn sub_assign(&mut self, other: &'a Self) {
        *self += &(-(*other));
    }
}

impl<T: Borrow<<Xsk233CurveConfig as CurveConfig>::ScalarField>> MulAssign<T> for Xsk233Projective {
    fn mul_assign(&mut self, other: T) {
        unsafe {
            let scalar_bytes = bigint_to_le_bytes(other.borrow().into_bigint());
            xsk233_mul_frob(
                &mut self.0,
                &self.0,
                scalar_bytes.as_ptr() as *const _,
                scalar_bytes.len(),
            );
        }
    }
}

impl<T: Borrow<<Xsk233CurveConfig as CurveConfig>::ScalarField>> Mul<T> for Xsk233Projective {
    type Output = Self;

    #[inline]
    fn mul(mut self, other: T) -> Self {
        self *= other;
        self
    }
}

impl From<Xsk233Affine> for Xsk233Projective {
    #[inline]
    fn from(p: Xsk233Affine) -> Xsk233Projective {
        Self(p.into_inner())
    }
}

impl CanonicalSerialize for Xsk233Projective {
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
    fn serialized_size(&self, compress: Compress) -> usize {
        Xsk233CurveConfig::serialized_size(compress)
    }
}

impl CanonicalDeserialize for Xsk233Projective {
    fn deserialize_with_mode<R: Read>(
        mut reader: R,
        compress: Compress,
        _validate: Validate,
    ) -> Result<Self, SerializationError> {
        if compress == Compress::No {
            return Err(SerializationError::IoError(io::Error::new(
                ErrorKind::Unsupported,
                "deserialization without compression is not supported",
            )));
        }

        let mut bytes = [0; 30];
        reader.read_exact(&mut bytes)?;

        unsafe {
            let mut result = xsk233_neutral;
            let success = xsk233_decode(&mut result, bytes.as_ptr() as *mut c_void);
            if success == 0 {
                return Err(SerializationError::IoError(io::Error::other(
                    "failed to deserialize",
                )));
            }

            Ok(Self(result))
        }
    }
}

impl Valid for Xsk233Projective {
    fn check(&self) -> Result<(), SerializationError> {
        self.into_affine().check()
    }

    fn batch_check<'a>(
        batch: impl Iterator<Item = &'a Self> + Send,
    ) -> Result<(), SerializationError>
    where
        Self: 'a,
    {
        let batch = batch.copied().collect::<Vec<_>>();
        let batch = Self::normalize_batch(&batch);
        Xsk233Affine::batch_check(batch.iter())
    }
}

impl<ConstraintF: Field> ToConstraintField<ConstraintF> for Xsk233Projective {
    #[inline]
    fn to_field_elements(&self) -> Option<Vec<ConstraintF>> {
        Xsk233Affine::from(*self).to_field_elements()
    }
}

impl ScalarMul for Xsk233Projective {
    type MulBase = Xsk233Affine;
    const NEGATION_IS_CHEAP: bool = true;

    fn batch_convert_to_mul_base(bases: &[Self]) -> Vec<Self::MulBase> {
        Self::normalize_batch(bases)
    }
}

impl VariableBaseMSM for Xsk233Projective {}

impl<T: Borrow<Xsk233Affine>> core::iter::Sum<T> for Xsk233Projective {
    fn sum<I: Iterator<Item = T>>(iter: I) -> Self {
        iter.fold(Xsk233Projective::zero(), |sum, x| sum + x.borrow())
    }
}
