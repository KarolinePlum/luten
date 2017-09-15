use std::collections::{HashMap, HashSet};


#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum WorkDay {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SlotRating {
    Good,
    Tolerable,
    NotFitting,
}

impl SlotRating {
    pub fn is_ok(&self) -> bool {
        match *self {
            SlotRating::NotFitting => false,
            _ => true,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Timeslot {
    pub day: WorkDay,
    pub slot_of_day: u16,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SlotAssignment {
    pub ratings: HashMap<Timeslot, SlotRating>,
}

impl SlotAssignment {
    pub fn new(good_slots: &[Timeslot], tolerable_slots: &[Timeslot]) -> Self {
        assert!(!good_slots.is_empty());

        let mut ratings = HashMap::new();
        ratings.extend(good_slots.iter().map(|&ts| (ts, SlotRating::Good)));
        ratings.extend(tolerable_slots.iter().map(|&ts| (ts, SlotRating::Tolerable)));

        // If this is false, there has been keys in both, `good_slots` and
        // `tolerable_slots`. This is not allowed.
        assert_eq!(good_slots.len() + tolerable_slots.len(), ratings.len());

        Self { ratings }
    }

    pub fn rating_for(&self, slot: Timeslot) -> SlotRating {
        self.ratings.get(&slot).cloned().unwrap_or(SlotRating::NotFitting)
    }

    pub fn intersect(&self, other: &Self) -> HashSet<Timeslot> {
        self.ratings.keys()
            .filter(|slot| other.ratings.contains_key(slot))
            .cloned()
            .collect()
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Tutor {
    pub name: String,
    pub slot_assignment: SlotAssignment,
    pub scale_factor: f32,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Student {
    pub name: String,
    pub slot_assignment: SlotAssignment,
    pub partner: Option<String>,
}


#[derive(Clone, PartialEq, Debug)]
pub struct Instance {
    pub students: Vec<Student>,
    pub tutors: Vec<Tutor>,
}

impl Instance {
    /// Returns all `Timeslot`s rated something other than `NotFitting` by at
    /// least one student or tutor
    pub fn slots(&self) -> Vec<Timeslot> {
        // student ratings
        let mut set: HashSet<Timeslot> = self.students.iter()
            .flat_map(|s| {
                s.slot_assignment.ratings.keys()
            }).cloned()
            .collect();
        // tutor ratings
        set.extend(self.tutors.iter().
            flat_map(|t| {
            t.slot_assignment.ratings.keys()
        }));
        set.into_iter().collect()
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Team<'a> {
    Single(&'a Student),
    Full(&'a Student, &'a Student),
}

impl<'a> Team<'a> {
    pub fn all_students<F>(&self, mut f: F) -> bool where
        F: FnMut(&Student) -> bool,
    {
        match *self {
            Team::Single(ref s) => f(s),
            Team::Full(ref s1, ref s2) => f(s1) && f(s2),
        }
    }

    pub fn contains(&self, s: &Student) -> bool {
        match *self {
            Team::Single(s1) => s1 == s,
            Team::Full(s1, s2) => s1 == s || s2 == s,
        }
    }

    pub fn members(&self) -> u8 {
        match *self {
            Team::Single(_) => 1,
            _ => 2,
        }
    }
    pub fn name(&self) -> String {
        match *self {
            Team::Single(s) => s.name.clone(),
            Team::Full(s1, s2) => {
                let mut x = s1.name.clone();
                x.push_str(&s2.name);
                x
            }
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Testat<'a> {
    pub slot: Timeslot,
    pub tutor: Tutor,
    pub team: Team<'a>
}

#[derive(Clone, PartialEq, Debug)]
pub struct Solution <'a> {
    pub testats: Vec<Testat<'a>>,
}
