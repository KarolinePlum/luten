use std::collections::HashMap;

use types::*;

const TESTATS_PER_STELLE: u16 = 1;

fn rating_value(rating: SlotRating) -> f32 {
    match rating {
        SlotRating::Good => 1.0,
        SlotRating::Tolerable => 0.5,
        _ => 0.0,
    }
}

fn number_of_alternatives(ratings: &SlotAssignment) -> f32 {
    ratings.ratings.values().map(|&r| rating_value(r)).sum()
}


pub fn team_up(students: &[Student]) -> Vec<Team> {
    let map: HashMap<&String, &Student> = students.iter()
        .map(|s| (&s.name, s))
        .collect();

    let mut teams: Vec<Team> = Vec::new();
    for s in students {
       let team = match s.partner {
            Some(ref name) => {
                // skip if the student was already added into a team together with his partner
                if teams.iter().any(|t| t.contains(s)) {
                    continue
                }
                let partner = map.get(name)
                    .expect("Instance contains Students with unknown partner");

                // assume that only symmetric partner wishes exist at this point
                assert_eq!(partner.partner, Some(s.name.clone()));

                Team::Full(s, partner)
            }
            _ => {
                Team::Single(s)
            }
        };
        teams.push(team);
    }
    teams
}

fn team_rating_values(team: &Team) -> HashMap<Timeslot, f32> {
    match *team {
        Team::Single(ref s) => {
            s.slot_assignment.ratings.iter().map(|(slot, &rating)| {
                (slot.clone(), rating_value(rating))
            }).collect()
        }
        Team::Full(ref s1, ref s2) => {
            s1.slot_assignment.intersect(&s2.slot_assignment)
                .iter()
                .map(|&slot| {
                    (slot, 0.5 * (rating_value(s1.slot_assignment.rating_for(slot))
                        + rating_value(s2.slot_assignment.rating_for(slot))))
                }).collect()
        }
    }
}

pub fn compute_demand(slot: Timeslot, teams: &[Team]) -> f32 {
    let mut demand = 0.0;
    for t in teams {
        let team_rating = team_rating_values(t);
        if let Some(value) = team_rating.get(&slot) {
            let alternatives: f32 = team_rating.values().sum();
            demand += value * t.members() as f32 / alternatives;
        }
    }
    demand
}

pub fn compute_supply(slot: Timeslot, tutors: &[Tutor]) -> f32 {
    let mut supply = 0.0;
    for t in tutors {
        let value = rating_value(t.slot_assignment.rating_for(slot));
        let expected_testats = t.scale_factor * (TESTATS_PER_STELLE as f32);
        let alternatives = number_of_alternatives(&t.slot_assignment);
        assert!(expected_testats <= alternatives);
        supply += (value * expected_testats) / alternatives;
    }
    supply
}

pub fn solve(_instance: &Instance) -> Solution {
    unimplemented!()
}
