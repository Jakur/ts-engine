#[derive(Clone, Copy, PartialEq)]
pub enum Effect {
    ShuttleDiplomacy,
    FormosanResolution,
    IronLady,
    VietnamRevolts,
    RedScarePurge,
    Containment,
    Brezhnev,
    CampDavid,
    AllowNato,
    DeGaulle,
    Nato,
    US_Hand_Revealed,
    USSR_Hand_Revealed,
    US_Japan,
    CubanMissileCrisis,
    NuclearSubs,
    Quagmire,
    SALT,
    BearTrap,
    NorthSeaOil,
    NoOpec,
}

impl Effect {
    pub fn permanent(&self) -> bool {
        use Effect::*;
        match self {
            ShuttleDiplomacy => true,
            FormosanResolution => true,
            IronLady => true,
            CampDavid => true,
            AllowNato => true,
            DeGaulle => true,
            Nato => true,
            BearTrap | Quagmire => true, // Can span multiple turns
            NoOpec => true,              // The lasting part of North Sea Oil
            _ => false,
        }
    }
}
