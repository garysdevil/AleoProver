use snarkvm_algorithms::{
    crypto_hash::PoseidonSponge,
    snark::marlin::{ahp::AHPForR1CS, FiatShamirAlgebraicSpongeRng, MarlinHidingMode, MarlinSNARK},
    SNARK,
};
use snarkvm_curves::bls12_377::{Bls12_377, Fq, Fr};
use snarkvm_fields::Field;
use snarkvm_r1cs::{errors::SynthesisError, ConstraintSynthesizer, ConstraintSystem};
use snarkvm_utilities::Uniform;

use rand::{self, thread_rng};

type MarlinInst = MarlinSNARK<Bls12_377, FS, MarlinHidingMode, [Fr]>;
type FS = FiatShamirAlgebraicSpongeRng<Fr, Fq, PoseidonSponge<Fq, 6, 1>>;

#[derive(Copy, Clone)]
pub struct Benchmark<F: Field> {
    pub a: Option<F>,
    pub b: Option<F>,
    pub num_constraints: usize,
    pub num_variables: usize,
}

impl<ConstraintF: Field> ConstraintSynthesizer<ConstraintF> for Benchmark<ConstraintF> {
    fn generate_constraints<CS: ConstraintSystem<ConstraintF>>(
        &self,
        cs: &mut CS,
    ) -> Result<(), SynthesisError> {
        let a = cs.alloc(|| "a", || self.a.ok_or(SynthesisError::AssignmentMissing))?;
        let b = cs.alloc(|| "b", || self.b.ok_or(SynthesisError::AssignmentMissing))?;
        let c = cs.alloc_input(
            || "c",
            || {
                let mut a = self.a.ok_or(SynthesisError::AssignmentMissing)?;
                let b = self.b.ok_or(SynthesisError::AssignmentMissing)?;

                a.mul_assign(&b);
                Ok(a)
            },
        )?;

        for i in 0..(self.num_variables - 3) {
            let _ = cs.alloc(
                || format!("var {}", i),
                || self.a.ok_or(SynthesisError::AssignmentMissing),
            )?;
        }

        for i in 0..(self.num_constraints - 1) {
            cs.enforce(
                || format!("constraint {}", i),
                |lc| lc + a,
                |lc| lc + b,
                |lc| lc + c,
            );
        }

        Ok(())
    }
}

pub fn snark_prove() {
    let num_constraints = 100;
    let num_variables = 100;
    let rng = &mut thread_rng();

    let x = Fr::rand(rng);
    let y = Fr::rand(rng);

    let max_degree = AHPForR1CS::<Fr, MarlinHidingMode>::max_degree(1000, 1000, 1000).unwrap();
    let universal_srs = MarlinInst::universal_setup(&max_degree, rng).unwrap();

    let circuit = Benchmark::<Fr> {
        a: Some(x),
        b: Some(y),
        num_constraints,
        num_variables,
    };

    let params = MarlinInst::circuit_setup(&universal_srs, &circuit).unwrap();

    MarlinInst::prove(&params.0, &circuit, rng).unwrap();
}
