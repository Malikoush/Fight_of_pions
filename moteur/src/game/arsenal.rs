use crate::Attack;

/// Un emplacement de l'arsenal d'un joueur : une attaque et sa recharge en cours.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArsenalSlot {
    attack: Attack,
    cooldown_left: u32,
}

impl ArsenalSlot {
    pub(crate) fn new(attack: Attack) -> ArsenalSlot {
        ArsenalSlot {
            attack,
            cooldown_left: 0,
        }
    }

    /// L'attaque de cet emplacement.
    pub fn attack(self) -> Attack {
        self.attack
    }

    /// Tours du joueur avant de pouvoir la relancer ; 0 si elle est prête.
    pub fn cooldown_left(self) -> u32 {
        self.cooldown_left
    }

    pub(crate) fn set_cooldown(&mut self, turns: u32) {
        self.cooldown_left = turns;
    }
}
