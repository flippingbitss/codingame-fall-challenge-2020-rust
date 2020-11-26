use std::{collections::{HashMap, HashSet, LinkedList, VecDeque}, io::{self, BufRead}, time::{Duration, Instant}};

macro_rules! parse_input {
    ($x:expr, $t:ident) => {
        $x.trim().parse::<$t>().unwrap()
    };
}

mod vec4 {
    use std::{
        fmt,
        ops::{Add, AddAssign, Sub, SubAssign},
    };

    #[derive(Eq, PartialEq, Copy, Clone, Hash)]
    pub struct Vec4 {
        pub x: i32,
        pub y: i32,
        pub z: i32,
        pub w: i32,
    }

    impl Vec4 {
        pub fn new(x: i32, y: i32, z: i32, w: i32) -> Vec4 {
            Vec4 { x, y, z, w }
        }

        pub fn zero() -> Vec4 {
            Vec4 {
                x: 0,
                y: 0,
                z: 0,
                w: 0,
            }
        }

        pub fn magnitude2(self) -> f64 {
            (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w) as f64
        }

        pub fn magnitude(self) -> f64 {
            f64::sqrt(self.magnitude2())
        }

        pub fn is_non_neg(self) -> bool {
            self.x >= 0 && self.y >= 0 && self.z >= 0 && self.w >= 0
        }
    }

    impl fmt::Debug for Vec4 {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            write!(fmt, "({}, {}, {}, {})", self.x, self.y, self.z, self.w)
        }
    }

    impl Add for Vec4 {
        type Output = Vec4;

        fn add(self, rhs: Self) -> Self::Output {
            Vec4 {
                x: self.x + rhs.x,
                y: self.y + rhs.y,
                z: self.z + rhs.z,
                w: self.w + rhs.w,
            }
        }
    }

    impl AddAssign for Vec4 {
        fn add_assign(&mut self, rhs: Self) {
            self.x += rhs.x;
            self.y += rhs.y;
            self.z += rhs.z;
            self.w += rhs.w;
        }
    }

    impl Sub for Vec4 {
        type Output = Vec4;

        fn sub(self, rhs: Self) -> Self::Output {
            Vec4 {
                x: self.x - rhs.x,
                y: self.y - rhs.y,
                z: self.z - rhs.z,
                w: self.w - rhs.w,
            }
        }
    }

    impl SubAssign for Vec4 {
        fn sub_assign(&mut self, rhs: Self) {
            self.x -= rhs.x;
            self.y -= rhs.y;
            self.z -= rhs.z;
            self.w -= rhs.w;
        }
    }
}

use vec4::Vec4;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum Action {
    Learn(i32),
    Brew(i32),
    Cast(i32),
    Rest,
    Wait,
}

impl ToString for Action {
    fn to_string(&self) -> String {
        match *self {
            Action::Learn(id) => format!("Learn {}", id),
            Action::Cast(id) => format!("Cast {}", id),
            Action::Brew(id) => format!("Brew {}", id),
            Action::Rest => "Rest".to_string(),
            Action::Wait => "Wait".to_string(),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
struct Spell {
    pub id: i32,
    pub is_repeatable: bool,
    pub is_castable: bool,
    pub delta: Vec4,
}

impl Spell {
    fn new(id: i32, is_repeatable: bool, is_castable: bool, delta: Vec4) -> Self {
        Self {
            id,
            is_repeatable,
            is_castable,
            delta,
        }
    }

    fn can_be_afforded_by(self, inventory: Vec4) -> bool {
        (inventory + self.delta).is_non_neg()
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
struct TomeSpell {
    pub id: i32,
    pub is_repeatable: bool,
    pub tome_index: i32,
    pub tax_count: i32,
    pub delta: Vec4,
}

impl TomeSpell {
    fn new(id: i32, is_repeatable: bool, tome_index: i32, tax_count: i32, delta: Vec4) -> Self {
        Self {
            id,
            is_repeatable,
            tome_index,
            tax_count,
            delta,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
struct Order {
    id: i32,
    price: i32,
    delta: Vec4,
}

impl Order {
    fn new(id: i32, price: i32, delta: Vec4) -> Self {
        Self { id, price, delta }
    }

    fn can_be_fulfilled_by(self, inventory: Vec4) -> bool {
        (inventory + self.delta).is_non_neg()
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
struct MagicTome {
    spells: Vec<TomeSpell>,
}

impl MagicTome {
    fn new(spells: Vec<TomeSpell>) -> Self {
        Self { spells }
    }

    fn add_spell(&mut self, spell: TomeSpell) {
        self.spells.push(spell)
    }

    fn remove_spell(&mut self, spell: TomeSpell) {
        if let Some(index) = self.spells.iter().position(|s| s.id == spell.id) {
            self.spells.remove(index);
        }
    }

    fn remove_spell_at(&mut self, index: usize) {
        self.spells.remove(index);
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
struct Player {
    score: i32,
    inventory: Vec4,
    spells: Vec<Spell>,
}

impl Player {
    fn new(score: i32, inventory: Vec4, spells: Vec<Spell>) -> Self {
        Self {
            score,
            inventory,
            spells,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
struct State {
    me: Player,
    orders: Vec<Order>,
    tome: MagicTome,
}
//
impl State {
    fn new(me: Player, tome: MagicTome, orders: Vec<Order>) -> Self {
        Self { me, tome, orders }
    }

    fn get_possible_actions(&self) -> Vec<Action> {
        let mut actions = Vec::new();
        let mut can_use_rest = false;

        // for spell in self.tome.spells.iter() {
        //     if self.me.inventory.x >= spell.tome_index {
        //         actions.push(Action::Learn(spell.id));
        //     }
        // }

        for order in self.orders.iter() {
            if order.can_be_fulfilled_by(self.me.inventory) {
                actions.push(Action::Brew(order.id));
            }
        }

        for spell in self.me.spells.iter() {
            if spell.is_castable {
                if spell.can_be_afforded_by(self.me.inventory) {
                    actions.push(Action::Cast(spell.id));
                }
            } else {
                can_use_rest = true;
            }
        }

        if can_use_rest {
            actions.push(Action::Rest);
        }
        // dbg!(&actions);
        actions
    }

    fn apply(&mut self, action: Action) {
        match action {
            Action::Brew(id) => {
                let order_idx = self
                    .orders
                    .iter()
                    .position(|o| o.id == id && o.can_be_fulfilled_by(self.me.inventory))
                    .unwrap();
                let order = self.orders[order_idx];

                self.me.inventory += order.delta;
                self.me.score += order.price;

                self.orders.remove(order_idx);
            }
            Action::Cast(id) => {
                if let Some(spell) = self.me.spells.iter_mut().find(|s| s.id == id) {
                    if spell.is_castable {
                        self.me.inventory += spell.delta;
                        spell.is_castable = false;
                    }
                    // dbg!(&self.me.spells);
                }
            }
            Action::Learn(id) => {
                if let Some(spell_idx) = self.tome.spells.iter().position(|s| s.id == id) {
                    let spell = self.tome.spells[spell_idx];
                    let delta = Vec4::new(spell.tome_index, 0, 0, 0);

                    if (self.me.inventory - delta).is_non_neg() {
                        let new_spell =
                            Spell::new(spell.id + 1000, spell.is_repeatable, true, spell.delta);
                        self.me.spells.push(new_spell);
                        self.tome.remove_spell(spell)
                    }
                }
            }
            Action::Rest => {
                for spell in self.me.spells.iter_mut() {
                    spell.is_castable = true;
                }
            }
            Action::Wait => {}
        }
    }

    fn find_brewable_order(&self) -> Option<Order> {
        self.orders
            .iter()
            .find(|o| o.can_be_fulfilled_by(self.me.inventory))
            .cloned()
    }

    fn read_from_io() -> Self {
        // std::fs::read("C:/Users/Matharu/Desktop/Development/Workspace/codingame/fall-challenge-2020-rust/input.txt");
        // let file = std::fs::File::open("C:/Users/Matharu/Desktop/Development/Workspace/codingame/fall-challenge-2020-rust/input.txt").unwrap();
        // let mut reader = io::BufReader::new(file);

        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let action_count = parse_input!(input_line, i32); // the number of spells and recipes in play

        let mut orders = Vec::with_capacity(5);
        let mut my_spells = Vec::new();
        let mut enemy_spells = Vec::new();
        let mut tome = Vec::new();

        for _ in 0..action_count as usize {
            let mut input_line = String::new();
            io::stdin().read_line(&mut input_line).unwrap();
            let inputs = input_line.split(" ").collect::<Vec<_>>();
            let action_id = parse_input!(inputs[0], i32); // the unique ID of this spell or recipe
            let action_type = inputs[1].trim().to_string(); // in the first league: BREW; later: CAST, OPPONENT_CAST, LEARN, BREW

            let x = parse_input!(inputs[2], i32); // tier-0 ingredient change
            let y = parse_input!(inputs[3], i32); // tier-1 ingredient change
            let z = parse_input!(inputs[4], i32); // tier-2 ingredient change
            let w = parse_input!(inputs[5], i32); // tier-3 ingredient change
            let delta = Vec4::new(x, y, z, w);
            let price = parse_input!(inputs[6], i32); // the price in rupees if this is a potion
            let tome_index = parse_input!(inputs[7], i32); // in the first two leagues: always 0; later: the index in the tome if this is a tome spell, equal to the read-ahead tax; For brews, this is the value of the current urgency bonus
            let tax_count = parse_input!(inputs[8], i32); // in the first two leagues: always 0; later: the amount of taxed tier-0 ingredients you gain from learning this spell; For brews, this is how many times you can still gain an urgency bonus
            let castable = parse_input!(inputs[9], i32) == 1; // in the first league: always 0; later: 1 if this is a castable player spell
            let repeatable = parse_input!(inputs[10], i32) == 1; // for the first two leagues: always 0; later: 1 if this is a repeatable player spell

            match action_type.as_str() {
                "BREW" => {
                    orders.push(Order::new(action_id, price, delta));
                }
                "CAST" => {
                    my_spells.push(Spell::new(action_id, repeatable, castable, delta));
                }
                "OPPONENT_CAST" => {
                    enemy_spells.push(Spell::new(action_id, repeatable, castable, delta));
                }
                "LEARN" => {
                    tome.push(TomeSpell::new(
                        action_id, repeatable, tome_index, tax_count, delta,
                    ));
                }
                _ => {}
            }
        }

        // for i in 0..2 as usize {
        let mut input_line = String::new();
        // io::stdin()
        io::stdin().read_line(&mut input_line).unwrap();
        let inputs = input_line.split(' ').collect::<Vec<_>>();
        let x = parse_input!(inputs[0], i32); // tier-0 ingredients in inventory
        let y = parse_input!(inputs[1], i32);
        let z = parse_input!(inputs[2], i32);
        let w = parse_input!(inputs[3], i32);
        let score = parse_input!(inputs[4], i32); // amount of rupees

        let me = Player::new(score, Vec4::new(x, y, z, w), my_spells);
        // }

        State::new(me, MagicTome::new(tome), orders)
    }
}

#[derive(Copy, Clone)]
struct Bot;

impl Bot {
    fn new() -> Self {
        Self {}
    }

    fn bfs(self, start_instant: &Instant, state: &State) -> Vec<Action> {
        const MAX_DURATION: Duration = Duration::from_millis(1000);
        let mut queue = LinkedList::<State>::new();
        // let mut queue = VecDeque::<State>::new();
        let mut visited = HashSet::<State>::new();
        let mut predecessor = HashMap::<State, State>::new();
        let mut pred_action = HashMap::<State, Action>::new();

        let initial_state = state.clone();
        queue.push_back(state.clone());
        pred_action.insert(state.clone(), Action::Wait);
        visited.insert(state.clone());
        let mut iterations = 0;
        while let Some(current_state) = queue.pop_front() {
            if start_instant.elapsed() > MAX_DURATION {
                // time over
                break;
            }
            iterations += 1;

            if current_state.find_brewable_order().is_some() {
                let mut path = Vec::<Action>::new();
                let mut curr_state = &current_state;
                while curr_state != &initial_state {
                    let action = pred_action.get(curr_state).expect("pred action not found");
                    let last_state = predecessor.get(curr_state).expect("prev state not found");
                    path.push(*action);
                    curr_state = last_state;
                }

                println!("{} game states visited", iterations);
                return path;
            }

            let curr_state = &current_state;

            for action in current_state.get_possible_actions() {
                let mut next = curr_state.clone();
                next.apply(action);

                if !visited.contains(&next) {
                    queue.push_back(next.clone());
                    pred_action.entry(next.clone()).or_insert(action);
                    predecessor
                        .entry(next.clone())
                        .or_insert(curr_state.clone());
                    visited.insert(next.clone());
                }
            }
        }

        Vec::new()
    }

    fn think(self, start_instant: &Instant, state: &State) -> Action {
        if let Some(order) = state.find_brewable_order() {
            return Action::Brew(order.id);
        }

        let actions: Vec<Action> = self.bfs(start_instant, state).into_iter().rev().collect();

        // for action in actions.iter() {
        //     println!("{}", action.to_string());
        // }

        if let Some(&action) = actions.first() {
            return action;
        }

        // println!("Time took: {} ms", start_instant.elapsed().as_millis());

        println!("no actions computed, taking first possible action");

        state.get_possible_actions().first().cloned().unwrap()
    }
}

fn main() {
    let bot = Bot::new();

    let state = State::read_from_io();

    let mut total_duration = Duration::new(0, 0);
    for _ in 0..100 {
        let turn_state = state.clone();
       // let turn_state = State::read_from_io();

        let start_instant = Instant::now();
        let action = bot.think(&start_instant, &turn_state);
        total_duration += start_instant.elapsed();

        println!("Action {}", action.to_string());
    }

     println!("Average time taken: {:?}", total_duration / 100)
}

fn main2() {
    let bot = Bot::new();

    let state = State::read_from_io();
    let mut turn = 0;
    let mut total_duration = Duration::new(0, 0);
    for _ in 0..100 {
        let turn_state = state.clone();
        let start_instant = Instant::now();

        let action = match turn {
            0..=6 => Action::Learn(turn_state.tome.spells.first().unwrap().id),
            _ => bot.think(&start_instant, &turn_state),
        };

        // let action = bot.think(&start_instant, &turn_state);
        total_duration += start_instant.elapsed();
        turn += 1;
        println!("Action {}", action.to_string());
    }

    println!("Average time taken: {:?}", total_duration / 101)
}
