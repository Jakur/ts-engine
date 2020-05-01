use crate::country::Side;

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
    AllowNato, // NATO
    DeGaulle,  // NATO
    Nato,      // NATO
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
    MissileEnvy,
    WWBY,
    WillyBrandt, // NATO
    TearDown,
    AWACS,
    FlowerPower,
    U2,
    AllowSolidarity,
    LatinAmericanPlus,
    LatinAmericanMinus,
}

impl Effect {
    pub fn allowed_side(&self) -> Side {
        use Effect::*;
        match self {
            ShuttleDiplomacy | FormosanResolution | IronLady | Containment | CampDavid
            | AllowNato | Nato | USSR_Hand_Revealed | US_Japan | NuclearSubs | Quagmire
            | NorthSeaOil | TearDown | AWACS | WWBY | AllowSolidarity => Side::US,

            VietnamRevolts | Brezhnev | DeGaulle | US_Hand_Revealed | BearTrap | NoOpec
            | WillyBrandt | FlowerPower | U2 => Side::USSR,

            RedScarePurge | CubanMissileCrisis | SALT | MissileEnvy | LatinAmericanPlus
            | LatinAmericanMinus => Side::Neutral,
        }
    }
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
            MissileEnvy => true,         // Can span multiple turns, technically
            WillyBrandt => true,
            FlowerPower => true,
            AllowSolidarity => true,
            _ => false,
        }
    }
}
