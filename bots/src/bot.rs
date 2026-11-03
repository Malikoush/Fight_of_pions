use fight_of_pions::{Action, Game, Rng};

/// Un joueur automatique.
///
/// Un bot est sans état : tout ce qu'il sait vient de la partie, et son hasard du
/// générateur que l'arène lui passe. Une partie entre bots se rejoue donc à l'identique
/// depuis sa graine. Sans état, il se partage entre plusieurs fils (`Sync`) : l'arène
/// joue plusieurs parties à la fois.
pub trait Bot: Sync {
    /// Le nom affiché dans les bilans de l'arène.
    fn name(&self) -> &'static str;

    /// L'action choisie maintenant pour le joueur qui doit décider, parmi celles que le
    /// moteur accepte. `None` seulement si la partie est finie.
    fn choose(&self, game: &Game, rng: &mut Rng) -> Option<Action>;
}
