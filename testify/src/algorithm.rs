use crate::chromosome::{ChromosomeGenerator, Chromosome};
use std::rc::Rc;
use std::collections::VecDeque;
use rand::random;
use std::ops::Deref;

#[derive(Debug)]
pub struct MOSA<G> where G: ChromosomeGenerator {
    population_size: u32,
    mutation_rate: f64,
    selection_rate: f64,
    chromosome_generator: Option<Box<G>>,
}

impl<G> MOSA<G> where G: ChromosomeGenerator {
    pub fn new() -> MOSA<G> {
        MOSA {
            population_size: 50,
            mutation_rate: 0.2,
            selection_rate: 0.3,
            chromosome_generator: None,
        }
    }

    pub fn chromosome_generator(&mut self, generator: G) -> &mut MOSA<G> {
        self.chromosome_generator = Some(Box::new(generator));
        self
    }

    pub fn population_size(&mut self, size: u32) -> &mut MOSA<G> {
        self.population_size = size;
        self
    }

    pub fn run(&self) {
        let mut current_generation = 0;
        let population = self.generate_random_population();
    }

    fn update_archive(&self) -> Vec<Box<G::C>> {
        unimplemented!()
    }

    fn generate_random_population(&self) -> Vec<Box<G::C>> {
        let chromosome_generator = self.chromosome_generator.as_ref().expect("Chromosome generator is not set.");
        let mut population = Vec::new();
        for _ in 0..self.population_size {
            population.push(chromosome_generator.generate());
        }

        population
    }
}

struct Archive<G, F> where G: ChromosomeGenerator, F: Fn(G::C) -> f64 {
    chromosomes: Vec<G::C>,
    objectives: Vec<Box<F>>,
}

impl<G, F> Archive<G, F> where G: ChromosomeGenerator, F: Fn(G::C) -> f64 {
    pub fn update_archive(&mut self, population: &Vec<G::C>) {}
}

pub struct SimulatedAnnealing<C> where C: Chromosome {
    initial_chromosome: Option<C>,
    max_transformations: u64,
    cooling_strategy: CoolingStrategy,
}

impl<C> SimulatedAnnealing<C> where C: Chromosome {
    pub fn new(cooling_strategy: CoolingStrategy) -> SimulatedAnnealing<C> {
        SimulatedAnnealing {
            initial_chromosome: None,
            max_transformations: 10,
            cooling_strategy,
        }
    }

    pub fn initial_chromosome(&mut self, chromosome: C) -> &mut Self {
        self.initial_chromosome = Some(chromosome);
        self
    }

    pub fn max_transformations(&mut self, transformations: u64) -> &mut SimulatedAnnealing<C> {
        self.max_transformations = transformations;
        self
    }

    fn acceptance_probability(&self, old_fitness: f64, new_fitness: f64) -> f64 {
        if new_fitness < old_fitness {
            1.0
        } else {
            (-(new_fitness - old_fitness / self.cooling_strategy.temperature).abs()).exp()
        }
    }

    pub fn run(&mut self) -> C {
        let mut current = Rc::new(self.initial_chromosome.take().unwrap());
        let mut transformations = 0;

        let mut current_fitness = current.calculate_fitness();
        let mut best = current.clone();
        let mut best_fitness = current_fitness;

        while transformations < self.max_transformations && self.cooling_strategy.improved_within_last_stages() {
            let neighbour = Rc::new(current.mutate());
            transformations += 1;

            let neighbour_fitness = neighbour.calculate_fitness();
            let acc_prob = self.acceptance_probability(current_fitness, neighbour_fitness);
            if acc_prob > random::<f64>() {
                current = neighbour.clone();
                current_fitness = current.calculate_fitness();

                println!("Current: {}, best: {}", current_fitness, best_fitness);
                if current_fitness < best_fitness {
                    println!("Better fitness: {}", current_fitness);
                    best = current.clone();
                    best_fitness = current_fitness;
                    self.cooling_strategy.energy_improved();
                }

                self.cooling_strategy.step(true);
            } else {
                self.cooling_strategy.step(false);
            }
        }
        println!("Done");

        (*best).clone()

        /*Rc::try_unwrap(best).unwrap_or_else(|err| {
            err
        })*/
    }
}

#[derive(Default)]
pub struct CoolingStrategy {
    pub degrees_of_freedom: u32,
    pub energy_history_size: usize,
    energy_history: VecDeque<bool>,
    acc_history: VecDeque<bool>,
    pub n_multiplicative: u32,
    pub n_max: u32,
    pub temperature: f64,
    accepted_n: u32,
    energy_improved: bool,
}

impl CoolingStrategy {
    pub fn new() -> CoolingStrategy {
        CoolingStrategy {
            degrees_of_freedom: 10,
            energy_history_size: 3,
            energy_history: Default::default(),
            acc_history: Default::default(),
            n_multiplicative: 0,
            n_max: 0,
            temperature: 0.0,
            accepted_n: 0,
            energy_improved: false,
        }
    }

    fn improved_within_last_stages(&self) -> bool {
        if self.energy_history.len() < self.energy_history_size {
            true
        } else {
            self.energy_history.iter().fold(false, |acc, &x| acc || x)
        }
    }

    fn reached_equilibrium(&self) -> bool {
        self.accepted_n >= self.n_multiplicative * self.degrees_of_freedom
    }

    fn energy_improved(&mut self) {
        self.energy_improved = true;
    }

    fn step(&mut self, candidate_accepted: bool) {
        self.add_accepted_history(candidate_accepted);

        if candidate_accepted && self.reached_equilibrium() {
            self.decrease_temperature(self.energy_improved);
            self.energy_improved = false;
        }
    }

    fn add_accepted_history(&mut self, candidate_accepted: bool) {
        self.acc_history.push_back(candidate_accepted);
        if candidate_accepted {
            self.accepted_n += 1;
        }

        if self.acc_history.len() > (self.n_max * self.degrees_of_freedom) as usize {
            if let Some(flag) = self.acc_history.pop_front() {
                if flag {
                    self.accepted_n -= 1;
                }
            }
        }
    }


    fn add_energy_history(&mut self, energy_improved: bool) {
        self.energy_history.push_back(energy_improved);

        if self.energy_history.len() > self.energy_history_size {
            self.energy_history.pop_front();
        }
    }

    fn decrease_temperature(&mut self, energy_improved: bool) {
        self.temperature *= 0.9;
        self.acc_history.clear();
        self.accepted_n = 0;
    }
}

#[derive(Debug)]
pub struct TemperatureStrategy {
    p0_range: f64,
    p0_min: f64,
    fitness_min: f64,
    fitness_range: f64,
}

impl TemperatureStrategy {
    pub fn new(p0_range: f64, p0_min: f64, fitness_min: f64, fitness_range: f64) -> TemperatureStrategy {
        TemperatureStrategy {
            p0_min,
            p0_range,
            fitness_range,
            fitness_min,
        }
    }

    pub fn init_temperature<C>(&self, start: C) -> (C, f64)
        where C: Chromosome {
        let mut current = start.clone();
        let initial_fitness = start.calculate_fitness();
        let mut current_fitness = current.calculate_fitness();
        let mut average_energy = 0.0;

        for i in 0..100 {
            let next = current.mutate();
            let next_fitness = next.calculate_fitness();
            average_energy += (next_fitness - current_fitness).abs();
            current_fitness = next_fitness;
            current = next;
        }

        average_energy /= 100.0;

        let p0 = (((initial_fitness - self.fitness_min) * self.p0_range) / self.fitness_range) + self.p0_min;
        let t0 = -(average_energy / p0.log10());

        (current, t0)
    }
}





