use ark_ec::CurveConfig;
use ark_ec::short_weierstrass::{Affine, SWCurveConfig};
use ark_ff::{Field, Fp256, MontBackend, MontConfig, MontFp};

#[derive(Clone, Default, PartialEq, Eq)]
pub struct Xsk233CurveConfig;

#[derive(MontConfig)]
#[modulus = "3450873173395281893717377931138512760570940988862252126328087024741343"]
#[generator = "3"]
pub struct FrConfig;
pub type Fr = Fp256<MontBackend<FrConfig, 4>>;

#[derive(MontConfig)]
#[modulus = "13803492693581127574869511724554050904902217944340773110325048447598591"]
#[generator = "4"]
pub struct FqConfig;
pub type Fq = Fp256<MontBackend<FqConfig, 4>>;

impl CurveConfig for Xsk233CurveConfig {
    type BaseField = Fq;
    type ScalarField = Fr;

    /// COFACTOR = 1
    const COFACTOR: &'static [u64] = &[0x4];

    /// COFACTOR_INV = COFACTOR^{-1} mod r = 1
    #[rustfmt::skip]
    const COFACTOR_INV: Fr = Fr::ONE;
}
impl SWCurveConfig for Xsk233CurveConfig {
    const COEFF_A: Fq = MontFp!("0");

    const COEFF_B: Fq = MontFp!("1");

    const GENERATOR: Affine<Self> = Affine::new_unchecked(G_GENERATOR_X, G_GENERATOR_Y);

    #[inline(always)]
    fn mul_by_a(x: Self::BaseField) -> Self::BaseField {
        x + x + x
    }
}

pub const G_GENERATOR_X: Fq =
    MontFp!("9980522611481012342443087688797002679043489582926858424680330554073382");

pub const G_GENERATOR_Y: Fq =
    MontFp!("12814767389816757102953168016268660157166792010263439198493421287958179");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::affine::Xsk233Affine;
    use crate::bigint_to_le_bytes;
    use crate::group::Xsk233Projective;
    use ark_ec::{AffineRepr, CurveGroup, VariableBaseMSM};
    use ark_ff::{AdditiveGroup, PrimeField};
    use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
    use ark_std::UniformRand;
    use rand::thread_rng;
    use std::hash::{DefaultHasher, Hash, Hasher};
    use std::io::Cursor;
    use xs233_sys::{
        xsk233_add, xsk233_double, xsk233_equals, xsk233_generator, xsk233_mul_frob, xsk233_neg,
        xsk233_neutral, xsk233_point,
    };

    fn rand_xsk233_sys_point(scalar: Fr) -> xsk233_point {
        let scalar_bytes = bigint_to_le_bytes(scalar.into_bigint());

        unsafe {
            let g = xsk233_generator;

            let mut res = xsk233_neutral;
            xsk233_mul_frob(
                &mut res,
                &g,
                scalar_bytes.as_ptr() as *const _,
                scalar_bytes.len(),
            );

            res
        }
    }

    fn rand_xsk233_ark_point(scalar: Fr) -> Xsk233Projective {
        Xsk233Affine::generator() * scalar
    }

    #[test]
    fn test_scalar_mul_correspondence() {
        unsafe {
            let mut rng = thread_rng();
            let scalar = Fr::rand(&mut rng);
            let scalar_bytes = bigint_to_le_bytes(scalar.into_bigint());

            let res_xsk = rand_xsk233_sys_point(scalar);
            let res_ark = rand_xsk233_ark_point(scalar);

            // Test projective structure multiplication
            let equals = xsk233_equals(res_ark.inner(), &res_xsk);
            assert!(equals != 0);

            // Test affine structure multiplication
            let res_ark_proj = res_ark.into_affine() * scalar;
            let mut res_xsk2 = xsk233_neutral;
            xsk233_mul_frob(
                &mut res_xsk2,
                &res_xsk,
                scalar_bytes.as_ptr() as *const _,
                scalar_bytes.len(),
            );

            let equals = xsk233_equals(res_ark_proj.inner(), &res_xsk2);
            assert!(equals != 0);
        }
    }

    #[test]
    fn test_addition_correspondence() {
        unsafe {
            let mut rng = thread_rng();
            let scalar1 = Fr::rand(&mut rng);
            let scalar2 = Fr::rand(&mut rng);

            let p1_xsk = rand_xsk233_sys_point(scalar1);
            let p2_xsk = rand_xsk233_sys_point(scalar2);

            let p1_ark = rand_xsk233_ark_point(scalar1);
            let p2_ark = rand_xsk233_ark_point(scalar2);

            let mut p12_xsk = xsk233_neutral;
            xsk233_add(&mut p12_xsk, &p1_xsk, &p2_xsk);

            let p12_ark = p1_ark + p2_ark;

            // Test projective structure addition
            let equals = xsk233_equals(p12_ark.inner(), &p12_xsk);
            assert!(equals != 0);

            // Test affine structure addition

            let mut p121_xsk = xsk233_neutral;
            xsk233_add(&mut p121_xsk, &p1_xsk, &p12_xsk);

            let p121_ark = p12_ark + p1_ark.into_affine();
            let equals_proj = xsk233_equals(p121_ark.inner(), &p121_xsk);
            assert!(equals_proj != 0);
        }
    }

    #[test]
    fn test_double_correspondence() {
        unsafe {
            let mut rng = thread_rng();
            let scalar1 = Fr::rand(&mut rng);

            let p1_xsk = rand_xsk233_sys_point(scalar1);
            let p1_ark = rand_xsk233_ark_point(scalar1);

            // Test double correspondence between ark_cc and xsk233_sys crates
            let mut p12_xsk = xsk233_neutral;
            xsk233_double(&mut p12_xsk, &p1_xsk);

            let p12_ark = p1_ark.double();

            let equals = xsk233_equals(p12_ark.inner(), &p12_xsk);
            assert!(equals != 0);

            // Test double(point) == point + point
            let equals = xsk233_equals(p12_ark.inner(), (p1_ark + p1_ark).inner());
            assert!(equals != 0);
        }
    }

    #[test]
    fn test_negation_correspondence() {
        unsafe {
            let mut rng = thread_rng();
            let scalar1 = Fr::rand(&mut rng);

            let p1_xsk = rand_xsk233_sys_point(scalar1);
            let p1_ark = rand_xsk233_ark_point(scalar1);

            let mut p1_neg_xsk = xsk233_neutral;
            xsk233_neg(&mut p1_neg_xsk, &p1_xsk);

            let p1_neg_ark = -p1_ark.into_affine();
            let equals = xsk233_equals(&p1_neg_xsk, p1_neg_ark.inner());
            assert!(equals != 0);
        }
    }

    #[test]
    fn test_msm() {
        unsafe {
            let mut rng = thread_rng();
            let scalar1 = Fr::rand(&mut rng);
            let scalar2 = Fr::rand(&mut rng);

            let g = Xsk233Affine::generator() * scalar1;
            let h = Xsk233Affine::generator() * scalar2;

            let msm1 = g + h;

            let msm2 = Xsk233Projective::msm(
                &[Xsk233Affine::generator(), Xsk233Affine::generator()],
                &[scalar1, scalar2],
            )
            .unwrap();

            let equals = xsk233_equals(msm1.inner(), msm2.inner());
            assert!(equals != 0);
        }
    }

    #[test]
    fn test_equality() {
        let mut rng = thread_rng();
        let scalar1 = Fr::rand(&mut rng);
        let scalar2 = Fr::rand(&mut rng);

        let g = Xsk233Affine::generator() * scalar1;
        let h = Xsk233Affine::generator() * scalar2;

        assert_eq!(g, g);
        assert_eq!(g, g.into_affine());
        assert_ne!(g, h);
    }

    #[test]
    fn test_hashing() {
        let scalar1 = Fr::from(100);
        let g = Xsk233Affine::generator() * scalar1;

        let mut hasher = DefaultHasher::new();
        g.hash(&mut hasher);

        assert_eq!(hasher.finish(), 15456673610726659490);
    }

    #[test]
    fn test_serialization() {
        let scalar1 = Fr::from(100);
        let g = Xsk233Affine::generator() * scalar1;

        let mut res = Vec::new();
        g.serialize_compressed(&mut res).unwrap();

        let g_deserialized = Xsk233Affine::deserialize_compressed(Cursor::new(res)).unwrap();

        assert_eq!(g, g_deserialized);
    }
}
