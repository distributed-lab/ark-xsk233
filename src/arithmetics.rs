// Implements AddAssign on Self by deferring to an implementation on &Self
#[macro_export]
macro_rules! impl_additive_ops_from_ref {
    ($type: ident) => {
        #[allow(unused_qualifications)]
        impl core::ops::Add<Self> for $type {
            type Output = Self;

            #[inline]
            fn add(self, other: Self) -> Self {
                let mut result = self;
                result.add_assign(&other);
                result
            }
        }

        #[allow(unused_qualifications)]
        impl<'a> core::ops::Add<&'a mut Self> for $type {
            type Output = Self;

            #[inline]
            fn add(self, other: &'a mut Self) -> Self {
                let mut result = self;
                result.add_assign(&*other);
                result
            }
        }

        impl<'b> core::ops::Add<$type> for &'b $type {
            type Output = $type;

            #[inline]
            fn add(self, mut other: $type) -> $type {
                other.add_assign(self);
                other
            }
        }

        #[allow(unused_qualifications)]
        impl<'a, 'b> core::ops::Add<&'a $type> for &'b $type {
            type Output = $type;

            #[inline]
            fn add(self, other: &'a $type) -> $type {
                let mut result = *self;
                result.add_assign(&*other);
                result
            }
        }

        #[allow(unused_qualifications)]
        impl<'a, 'b> core::ops::Add<&'a mut $type> for &'b $type {
            type Output = $type;

            #[inline]
            fn add(self, other: &'a mut $type) -> $type {
                let mut result = *self;
                result.add_assign(&*other);
                result
            }
        }

        impl<'b> core::ops::Sub<$type> for &'b $type {
            type Output = $type;

            #[inline]
            fn sub(self, other: $type) -> $type {
                let mut result = *self;
                result.sub_assign(&other);
                result
            }
        }

        #[allow(unused_qualifications)]
        impl<'a, 'b> core::ops::Sub<&'a $type> for &'b $type {
            type Output = $type;

            #[inline]
            fn sub(self, other: &'a $type) -> $type {
                let mut result = *self;
                result.sub_assign(&*other);
                result
            }
        }

        #[allow(unused_qualifications)]
        impl<'a, 'b> core::ops::Sub<&'a mut $type> for &'b $type {
            type Output = $type;

            #[inline]
            fn sub(self, other: &'a mut $type) -> $type {
                let mut result = *self;
                result.sub_assign(&*other);
                result
            }
        }

        #[allow(unused_qualifications)]
        impl core::ops::Sub<Self> for $type {
            type Output = Self;

            #[inline]
            fn sub(self, other: Self) -> Self {
                let mut result = self;
                result.sub_assign(&other);
                result
            }
        }

        #[allow(unused_qualifications)]
        impl<'a> core::ops::Sub<&'a mut Self> for $type {
            type Output = Self;

            #[inline]
            fn sub(self, other: &'a mut Self) -> Self {
                let mut result = self;
                result.sub_assign(&*other);
                result
            }
        }

        #[allow(unused_qualifications)]
        impl core::iter::Sum<Self> for $type {
            fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
                iter.fold(Self::zero(), core::ops::Add::add)
            }
        }

        #[allow(unused_qualifications)]
        impl<'a> core::iter::Sum<&'a Self> for $type {
            fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
                iter.fold(Self::zero(), core::ops::Add::add)
            }
        }

        #[allow(unused_qualifications)]
        impl core::ops::AddAssign<Self> for $type {
            fn add_assign(&mut self, other: Self) {
                self.add_assign(&other)
            }
        }

        #[allow(unused_qualifications)]
        impl core::ops::SubAssign<Self> for $type {
            fn sub_assign(&mut self, other: Self) {
                self.sub_assign(&other)
            }
        }

        #[allow(unused_qualifications)]
        impl<'a> core::ops::AddAssign<&'a mut Self> for $type {
            fn add_assign(&mut self, other: &'a mut Self) {
                self.add_assign(&*other)
            }
        }

        #[allow(unused_qualifications)]
        impl<'a> core::ops::SubAssign<&'a mut Self> for $type {
            fn sub_assign(&mut self, other: &'a mut Self) {
                self.sub_assign(&*other)
            }
        }
    };
}
