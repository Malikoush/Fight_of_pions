/// Générateur pseudo-aléatoire déterministe (SplitMix64), sans dépendance.
///
/// Une même graine donne toujours la même suite, sur toutes les machines et toutes les
/// versions de Rust : c'est ce qui rend les parties, les tests et les bots reproductibles.
/// Tout le hasard du jeu passe par lui ; il n'est pas fait pour la cryptographie.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Rng {
    state: u64,
}

impl Rng {
    /// Un générateur initialisé avec cette graine.
    pub fn new(seed: u64) -> Rng {
        Rng { state: seed }
    }

    /// Le prochain nombre de la suite, sur 64 bits.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Un nombre tiré uniformément dans `0..bound`, ou `None` si `bound` vaut 0.
    pub fn below(&mut self, bound: u64) -> Option<u64> {
        if bound == 0 {
            return None;
        }
        // On rejette le haut de la plage, qui ne contient pas un nombre entier de fois
        // `bound` : sinon les petits résultats sortiraient un peu plus souvent.
        let zone = u64::MAX - u64::MAX % bound;
        loop {
            let value = self.next_u64();
            if value < zone {
                return Some(value % bound);
            }
        }
    }

    /// Un élément de la liste tiré uniformément, ou `None` si elle est vide.
    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> Option<&'a T> {
        let len = u64::try_from(items.len()).ok()?;
        let index = usize::try_from(self.below(len)?).ok()?;
        items.get(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Valeurs de référence de l'implémentation C d'origine de SplitMix64 (graine 0).
    #[test]
    fn matches_the_reference_sequence() {
        let mut rng = Rng::new(0);
        assert_eq!(rng.next_u64(), 0xE220_A839_7B1D_CDAF);
        assert_eq!(rng.next_u64(), 0x6E78_9E6A_A1B9_65F4);
        assert_eq!(rng.next_u64(), 0x06C4_5D18_8009_454F);
    }

    #[test]
    fn same_seed_gives_same_sequence() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
        assert_ne!(Rng::new(1).next_u64(), Rng::new(2).next_u64());
    }

    #[test]
    fn below_stays_in_range_and_reaches_every_value() {
        let mut rng = Rng::new(7);
        let mut seen = [false; 6];
        for _ in 0..1000 {
            let value = rng.below(6).unwrap();
            assert!(value < 6);
            seen[value as usize] = true;
        }
        assert!(seen.iter().all(|&s| s));
        assert_eq!(rng.below(0), None);
    }

    #[test]
    fn pick_returns_an_element_or_nothing() {
        let mut rng = Rng::new(3);
        let items = ['a', 'b', 'c'];
        assert!(items.contains(rng.pick(&items).unwrap()));
        assert_eq!(rng.pick::<char>(&[]), None);
    }
}
