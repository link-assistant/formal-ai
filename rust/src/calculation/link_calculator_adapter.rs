//! Delegate expressions and supply verified rates only for currency operands.

use super::{ArithmeticError, CalculationEngine, CalculationEvaluation};

pub(super) fn evaluate(
    expression: &str,
    supply_rates: impl FnOnce(&mut link_calculator::Calculator, &[String]),
) -> Result<CalculationEvaluation, ArithmeticError> {
    // A bare `.` (`2. 3`) is rejected by link-calculator itself as a
    // recoverable parse error since 0.18.0 (link-assistant/calculator#168), so
    // it falls through to the local evaluator like any other upstream error.
    // The default parser has an empty rate database. Published currency quotes
    // are resolved by the solver, so a failed operand can never use sample rates.
    let mut calculator = link_calculator::Calculator::default();
    let evaluated = calculator
        .calculate_with_value(expression)
        .or_else(|error| {
            let plan = calculator.plan_internal(expression);
            if plan.currencies.len() < 2 {
                return Err(error);
            }
            supply_rates(&mut calculator, &plan.currencies);
            calculator.calculate_with_value(expression)
        });
    let (_expression, value, steps, lino) =
        evaluated.map_err(|error| ArithmeticError::Calculator(error.to_string()))?;
    Ok(CalculationEvaluation {
        formatted: value.to_display_string(),
        engine: CalculationEngine::LinkCalculator,
        lino: Some(lino),
        steps,
    })
}
