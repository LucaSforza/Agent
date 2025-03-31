use crate::problem::{CostructSolution, Problem, SuitableState};
enum Domain {
    Int(i32),
    Float(f64),
    Str(String),
}

pub trait Variable {
    type IndexVar;

    fn set_val(&mut self, i: Self::IndexVar, val: Domain);
    fn get_val(&self, i: Self::IndexVar) -> Domain;
}

pub trait ConstraintSatisfaction: Problem + SuitableState {
    type ChooseItem;
}
pub trait CSPAlgoritm {
    fn resolve();
}
pub struct BackTracking<P: ConstraintSatisfaction> {
    stack: Vec<P::ChooseItem>,
}

impl<P> CSPAlgoritm for BackTracking<P>
where
    P: ConstraintSatisfaction,
{
    fn resolve() {
        todo!()
    }
}
