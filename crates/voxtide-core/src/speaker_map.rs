use std::collections::BTreeMap;

// 26 chips before any wrap: the old 4-letter table gave the 5th speaker 'A'
// again, fusing two different people under chip-equality merging — and only
// the wrapped letter is persisted, so the fusion was unrecoverable. The
// modulo below stays for the pathological >26 case (reuse beats panicking).
const LETTERS: &[char] = &[
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];

#[derive(Debug, Clone, Default)]
pub struct SpeakerMap {
    order: Vec<String>,
    map: BTreeMap<String, char>,
}

impl SpeakerMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_snapshot(snapshot: BTreeMap<String, char>) -> Self {
        let mut order: Vec<(String, char)> =
            snapshot.iter().map(|(k, v)| (k.clone(), *v)).collect();
        order.sort_by_key(|(_, v)| *v);
        Self {
            order: order.into_iter().map(|(k, _)| k).collect(),
            map: snapshot,
        }
    }

    pub fn snapshot(&self) -> BTreeMap<String, char> {
        self.map.clone()
    }

    pub fn chip_for(&mut self, soniox_id: &str) -> char {
        if let Some(c) = self.map.get(soniox_id) {
            return *c;
        }
        let idx = self.order.len() % LETTERS.len();
        let letter = LETTERS[idx];
        self.order.push(soniox_id.to_string());
        self.map.insert(soniox_id.to_string(), letter);
        letter
    }
}
