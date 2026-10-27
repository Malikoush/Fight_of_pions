//! Assemble les morceaux du labo et fait circuler les messages à chaque image :
//! l'affichage produit des `Intention`, l'application en fait des `Demande` à la partie
//! ou des gestes de sélection, et le panneau reçoit les `Nouvelle`.
//!
//! Deux écrans : **Partie** (jouer, seul, contre un bot ou regarder deux bots) et
//! **Arène** (simuler des parties entre bots, et en revoir une dans l'écran Partie).
//!
//! Choisir une attaque se fait sur une copie de la partie (`Partie::apercu`) : seule la
//! suite complète (attaque, lanceur, cibles) est jouée, d'un bloc. Annuler est gratuit.

use crate::arene::EcranArene;
use crate::intention::Intention;
use crate::panneau::Panneau;
use crate::partie::{Demande, Nouvelle, Partie, en_plein_choix};
use crate::{plateau, theme};
use bots::BotKind;
use eframe::egui;
use fight_of_pions::{Action, GameConfig, GameError, Position};
use std::time::Instant;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Ecran {
    Partie,
    Arene,
}

pub struct Labo {
    ecran: Ecran,
    partie: Partie,
    panneau: Panneau,
    arene: EcranArene,
    /// L'attaque en cours de choix dans l'interface, pas encore jouée.
    selection: Option<Vec<Action>>,
    /// Le moment du dernier coup joué : un bot attend un peu avant de jouer le sien.
    dernier_coup: Instant,
}

impl Labo {
    pub fn new(config: GameConfig) -> Result<Self, GameError> {
        Ok(Self {
            ecran: Ecran::Partie,
            partie: Partie::new(config)?,
            panneau: Panneau::new(),
            arene: EcranArene::new(),
            selection: None,
            dernier_coup: Instant::now(),
        })
    }

    /// Transmet une demande à la partie et ses nouvelles au panneau. Vrai si le moteur a
    /// refusé quelque chose.
    fn demander(&mut self, demande: Demande) -> bool {
        self.selection = None;
        self.dernier_coup = Instant::now();
        let mut refus = false;
        for nouvelle in self.partie.traiter(demande) {
            refus |= matches!(nouvelle, Nouvelle::Refus(_));
            self.panneau.recevoir(&nouvelle);
        }
        refus
    }

    /// Le bot qui doit décider maintenant, s'il y en a un.
    fn bot_qui_doit_jouer(&self) -> Option<BotKind> {
        let decision = self.partie.game().pending_decision()?;
        self.panneau.controleur(decision.player())
    }

    /// En lecture automatique, le bot qui doit jouer joue un coup une fois son attente
    /// écoulée ; sinon on revient voir à la fin de l'attente.
    fn faire_jouer_les_bots(&mut self, ctx: &egui::Context) {
        let Some(sorte) = self.bot_qui_doit_jouer() else {
            return;
        };
        // Une attaque qu'un humain choisissait n'a plus de sens si un bot a pris sa place.
        self.selection = None;
        let Some(attente) = self.panneau.attente_des_bots() else {
            return;
        };
        let ecoule = self.dernier_coup.elapsed();
        if ecoule < attente {
            ctx.request_repaint_after(attente - ecoule);
            return;
        }
        // Un coup refusé est un bug du bot : on arrête la lecture plutôt que de le
        // retenter à chaque image.
        if self.demander(Demande::CoupDuBot(sorte)) {
            self.panneau.arreter_la_lecture();
        }
        ctx.request_repaint();
    }

    /// Une case cliquée : pose hors attaque, ou prochain choix de l'attaque en cours.
    fn choisir_case(&mut self, pos: Position) {
        let Some(mut suite) = self.selection.take() else {
            // Sans sélection : une pose, ou un choix si la vraie partie en attend un
            // (partie chargée au milieu d'une attaque, par exemple).
            let action = if en_plein_choix(self.partie.game()) {
                Action::Choose(pos)
            } else {
                Action::Place(pos)
            };
            self.demander(Demande::Agir(action));
            return;
        };
        suite.push(Action::Choose(pos));
        match self.partie.apercu(&suite) {
            // Le choix est accepté et l'attaque attend encore un choix : on continue.
            Ok(apercu) if en_plein_choix(&apercu) => self.selection = Some(suite),
            // Le dernier choix est fait : l'attaque entière est jouée pour de vrai.
            Ok(_) => {
                self.demander(Demande::Jouer(suite));
            }
            // Choix refusé : on l'explique, et la sélection reste où elle était.
            Err(e) => {
                suite.pop();
                self.selection = Some(suite);
                self.panneau.recevoir(&Nouvelle::Refus(e));
            }
        }
    }

    fn ecran_partie(&mut self, ui: &mut egui::Ui, intentions: &mut Vec<Intention>) {
        // Pendant le choix d'une attaque, on montre la copie : lanceur et cibles marqués.
        let apercu = self
            .selection
            .as_ref()
            .and_then(|suite| self.partie.apercu(suite).ok());
        let affichee = apercu.as_ref().unwrap_or(self.partie.game());
        // Le panneau latéral doit être montré avant le plateau, qui prend la place restante.
        self.panneau.show(
            ui,
            self.partie.game(),
            self.partie.record(),
            self.selection.is_some(),
            intentions,
        );
        plateau::show(ui, affichee, intentions);
    }

    fn traiter(&mut self, intention: Intention) {
        // Au tour d'un bot, les gestes de jeu d'un humain sont refusés : le bot joue seul,
        // ou avec « Coup du bot ». Les outils (annuler, hasard, charger…) restent permis.
        let geste_de_jeu = matches!(
            intention,
            Intention::CommencerAttaque(_)
                | Intention::ChoisirCase(_)
                | Intention::Demande(Demande::Agir(_) | Demande::Jouer(_))
        );
        if geste_de_jeu && self.bot_qui_doit_jouer().is_some() {
            self.panneau
                .signaler("C'est au bot de jouer : change le contrôleur pour jouer à sa place.");
            return;
        }
        match intention {
            Intention::Demande(demande) => {
                self.demander(demande);
            }
            Intention::CommencerAttaque(slot) => {
                self.selection = Some(vec![Action::UseAttack(slot)]);
            }
            Intention::ChoisirCase(pos) => self.choisir_case(pos),
            Intention::AnnulerSelection => self.selection = None,
            Intention::Revoir {
                config,
                graine,
                bots,
            } => {
                self.panneau.confier(bots.map(Some));
                // « Nouvelle partie » relancera la même configuration, avec une autre graine.
                self.panneau.preparer_la_prochaine(&config);
                self.demander(Demande::NouvellePartie {
                    config,
                    graine: Some(graine),
                });
                self.ecran = Ecran::Partie;
            }
        }
    }
}

impl eframe::App for Labo {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("ecrans").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.ecran, Ecran::Partie, theme::ECRAN_PARTIE);
                ui.selectable_value(&mut self.ecran, Ecran::Arene, theme::ECRAN_ARENE);
            });
        });

        let mut intentions = Vec::new();
        match self.ecran {
            Ecran::Partie => self.ecran_partie(ui, &mut intentions),
            Ecran::Arene => self
                .arene
                .show(ui, self.partie.game().config(), &mut intentions),
        }

        if !intentions.is_empty() {
            for intention in intentions {
                self.traiter(intention);
            }
            // L'image vient d'être dessinée avec l'ancien état : on en redemande une.
            ui.ctx().request_repaint();
        }
        // Les bots ne jouent que sous les yeux de l'utilisateur.
        if self.ecran == Ecran::Partie {
            self.faire_jouer_les_bots(ui.ctx());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::partie::config_initiale;
    use fight_of_pions::Player;

    /// Un labo où le joueur 1 est un bot et le joueur 2 un humain.
    fn labo_face_a_un_bot() -> Labo {
        let mut labo = Labo::new(config_initiale()).unwrap();
        labo.panneau.confier([Some(BotKind::Greedy), None]);
        labo
    }

    #[test]
    fn au_tour_d_un_bot_les_gestes_de_jeu_humains_sont_refuses() {
        let mut labo = labo_face_a_un_bot();
        let case = Position { row: 4, col: 4 };
        for geste in [
            Intention::ChoisirCase(case),
            Intention::CommencerAttaque(0),
            Intention::Demande(Demande::Agir(Action::Place(case))),
            Intention::Demande(Demande::Agir(Action::EndTurn)),
        ] {
            labo.traiter(geste);
        }
        assert!(labo.partie.record().actions.is_empty());
        assert!(labo.selection.is_none());
    }

    #[test]
    fn revoir_regle_la_partie_les_bots_et_l_ecran() {
        let mut labo = labo_face_a_un_bot();
        labo.ecran = Ecran::Arene;
        let config = bots::variants()[1].config.clone();
        let bots = [BotKind::Banker, BotKind::Random];
        labo.traiter(Intention::Revoir {
            config: config.clone(),
            graine: 7,
            bots,
        });
        assert!(labo.ecran == Ecran::Partie);
        assert_eq!(labo.partie.game().seed(), 7);
        assert_eq!(labo.partie.game().config(), &config);
        assert_eq!(labo.panneau.controleur(Player::One), Some(BotKind::Banker));
        assert_eq!(labo.panneau.controleur(Player::Two), Some(BotKind::Random));
        assert!(labo.panneau.attente_des_bots().is_some());
    }
}
