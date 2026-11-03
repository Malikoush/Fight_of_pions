use crate::Attack;

/// Un emplacement de l'arsenal d'un joueur : une attaque, sa recharge en cours, et si elle
/// est encore secrète (sa cache, pas encore jouée).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ArsenalSlot {
    attack: Attack,
    cooldown_left: u32,
    secret: bool,
}

impl ArsenalSlot {
    pub(crate) fn new(attack: Attack) -> ArsenalSlot {
        ArsenalSlot {
            attack,
            cooldown_left: 0,
            secret: false,
        }
    }

    /// Une carte de la cache secrète : l'adversaire ne la voit pas avant qu'elle soit
    /// jouée.
    pub(crate) fn secret(attack: Attack) -> ArsenalSlot {
        ArsenalSlot {
            secret: true,
            ..ArsenalSlot::new(attack)
        }
    }

    /// Vrai si l'adversaire de son propriétaire ne la connaît pas encore : une carte de sa
    /// cache, jamais jouée. Un client ne la montre qu'à son propriétaire
    /// ([`crate::Game::arsenal_seen_by`]).
    pub fn is_secret(self) -> bool {
        self.secret
    }

    pub(crate) fn reveal(&mut self) {
        self.secret = false;
    }

    pub(crate) fn replace_attack(&mut self, attack: Attack) {
        self.attack = attack;
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
