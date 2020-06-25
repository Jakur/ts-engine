use super::Card;
use crate::country::Side;

#[derive(Clone, Copy, PartialEq, Debug)]
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
    TerrorismPlus,
    Reformer,
    IranContra,
    EvilEmpire,
    AldrichAmes,         // More powerful hand reveal
    US_Scoring_Revealed, // Least powerful hand reveal
    Norad,
    Yuri,
}

impl Effect {
    pub fn allowed_side(&self) -> Side {
        use Effect::*;
        match self {
            ShuttleDiplomacy | FormosanResolution | IronLady | Containment | CampDavid
            | AllowNato | Nato | USSR_Hand_Revealed | US_Japan | NuclearSubs | Quagmire
            | NorthSeaOil | TearDown | AWACS | WWBY | AllowSolidarity | EvilEmpire | Norad => {
                Side::US
            }

            VietnamRevolts | Brezhnev | DeGaulle | US_Hand_Revealed | BearTrap | NoOpec
            | WillyBrandt | FlowerPower | U2 | TerrorismPlus | Reformer | IranContra
            | AldrichAmes | US_Scoring_Revealed | Yuri => Side::USSR,

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
            TerrorismPlus => true,
            Reformer => true,
            _ => false,
        }
    }
    pub fn card_to_effect(card: Card) -> Option<Self> {
        use Effect::*;
        let effect = match card {
            Card::Formosan_Resolution => FormosanResolution,
            Card::Shuttle_Diplomacy => ShuttleDiplomacy,
            Card::The_Iron_Lady => IronLady,
            Card::Vietnam_Revolts => VietnamRevolts,
            Card::Red_Scare_Purge => RedScarePurge,
            Card::Containment => Containment,
            Card::Camp_David_Accords => CampDavid,
            Card::Brezhnev_Doctrine => Brezhnev,
            Card::Warsaw_Pact_Formed | Card::Marshall_Plan => AllowNato,
            Card::De_Gaulle_Leads_France => DeGaulle, // NATO
            Card::NATO => Nato,                       // NATO
            Card::Lone_Gunman => US_Hand_Revealed,
            Card::CIA_Created => USSR_Hand_Revealed,
            Card::US_Japan_Mutual_Defense_Pact => US_Japan,
            Card::Cuban_Missile_Crisis => CubanMissileCrisis,
            Card::Nuclear_Subs => NuclearSubs,
            Card::Quagmire => Quagmire,
            Card::SALT_Negotiations => SALT,
            Card::Bear_Trap => BearTrap,
            Card::North_Sea_Oil => NorthSeaOil, // Todo NO OPEC
            Card::Missile_Envy => MissileEnvy,
            Card::We_Will_Bury_You => WWBY,
            Card::Willy_Brandt => WillyBrandt, // NATO
            Card::Tear_Down_This_Wall => TearDown,
            Card::AWACS => AWACS,
            Card::Flower_Power => FlowerPower,
            Card::U2_Incident => U2,
            Card::John_Paul => AllowSolidarity,
            // TODO LADS
            Card::Terrorism => TerrorismPlus,
            Card::The_Reformer => Reformer,
            Card::Iran_Contra_Scandal => IranContra,
            Card::An_Evil_Empire => EvilEmpire,
            Card::Aldrich_Ames_Remix => AldrichAmes, // More powerful hand reveal
            Card::The_Cambridge_Five => US_Scoring_Revealed, // Least powerful hand reveal
            Card::NORAD => Norad,
            Card::Yuri_And_Samantha => Yuri,
            _ => return None,
        };
        Some(effect)
    }
}
