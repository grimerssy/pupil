use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Percentile(u8);

impl Percentile {
    pub fn new(percentile: Decimal) -> anyhow::Result<Self> {
        if percentile < Decimal::ZERO || percentile > Decimal::ONE {
            anyhow::bail!("percentile {percentile} is out of range");
        }
        let percent = u8::try_from(percentile * Decimal::ONE_HUNDRED).unwrap();
        Ok(Self(percent))
    }
}
