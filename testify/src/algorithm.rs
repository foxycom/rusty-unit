use crate::chromosome::{Chromosome, FitnessValue, TestCase};
use crate::generators::TestIdGenerator;
use crate::operators::RankSelection;
use crate::selection::Selection;
use crate::source::{Branch, BranchManager, SourceFile};
use pbr::ProgressBar;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Write;
use std::iter::FromIterator;
use std::option::Option::Some;
use std::rc::Rc;
use std::time::Instant;

pub struct DynaMOSA<S: Selection> {
    population_size: u64,
    mutation_rate: f64,
    selection_rate: f64,
    crossover_rate: f64,
    generations: u64,
    branch_manager: Rc<RefCell<BranchManager>>,
    selection: S,
}

impl<S: Selection> DynaMOSA<S> {
    pub fn new(
        population_size: u64,
        mutation_rate: f64,
        selection_rate: f64,
        crossover_rate: f64,
        generations: u64,
        branch_manager: Rc<RefCell<BranchManager>>,
        selection: S,
    ) -> Self {
        DynaMOSA {
            population_size,
            mutation_rate,
            selection_rate,
            crossover_rate,
            generations,
            branch_manager,
            selection,
        }
    }

    pub fn set_population_size(&mut self, population_size: u64) {
        self.population_size = population_size;
    }
    pub fn set_mutation_rate(&mut self, mutation_rate: f64) {
        self.mutation_rate = mutation_rate;
    }
    pub fn set_selection_rate(&mut self, selection_rate: f64) {
        self.selection_rate = selection_rate;
    }
    pub fn set_crossover_rate(&mut self, crossover_rate: f64) {
        self.crossover_rate = crossover_rate;
    }
    pub fn set_generations(&mut self, generations: u64) {
        self.generations = generations;
    }
    pub fn run<C: Chromosome>(
        &mut self,
        mut source_file: SourceFile,
        initial_population: Vec<C>,
    ) -> Result<C, Box<dyn Error>> {
        let mut archive = Archive::new(self.branch_manager.clone());

        let count = (self.generations + 1) * self.population_size;
        let mut pb = ProgressBar::new(count);
        pb.format("╢▌▌░╟");

        let mut current_generation = 0;

        let mut population = initial_population;

        source_file.add_tests(&population, true);
        source_file.run_tests(&mut population);
        source_file.clear_tests();

        pb.add(self.population_size);

        archive.update(&population);

        while current_generation < self.generations {
            self.branch_manager
                .borrow_mut()
                .set_current_population(&population);

            let mut offspring = self.generate_offspring(&population)?;

            source_file.add_tests(&offspring, true);
            source_file.run_tests(&mut offspring);
            source_file.clear_tests();

            archive.update(&population);

            offspring.append(&mut population);

            // TODO there is a bug in sort, duplicate ids
            let mut fronts =
                PreferenceSorter::sort(&offspring, self.branch_manager.borrow().branches());

            for i in 0..fronts.len() {
                let front = fronts.get(&i).unwrap();
                let front = SVD::compute(front, self.branch_manager.borrow().branches())?;
                for t in &front {
                    population.push(t.clone());
                    if population.len() == self.population_size as usize {
                        break;
                    }
                }

                if population.len() == self.population_size as usize {
                    break;
                }
            }

            pb.add(self.population_size);
            current_generation += 1;
        }

        source_file.add_tests(&population, true);

        let mut tmp_file = File::create("fitness.log").unwrap();

        population.iter().for_each(|t| {
            let bm = self.branch_manager.borrow();
            let branches = bm.branches();
            let fitness = branches
                .iter()
                .map(|b| format!("b = {}, f = {}", b.id(), b.fitness(t)))
                .fold(String::new(), |acc, b| acc + &b.to_string() + ", ");
            let line = format!("Test {} => ({})\n", t.id(), fitness);
            tmp_file.write_all(&line.as_bytes());
        });

        let (uncovered_branches, coverage) = self.coverage();
        Ok(TestSuite {
            uncovered_branches,
            coverage,
            tests: population,
        })
    }
}

#[derive(Debug)]
pub struct MOSA {
    population_size: u64,
    mutation_rate: f64,
    selection_rate: f64,
    crossover_rate: f64,
    branch_manager: Rc<RefCell<BranchManager>>,
    generations: u64,
    rank_selection: RankSelection,
}

impl MOSA {
    pub fn new(rank_selection: RankSelection, branch_manager: Rc<RefCell<BranchManager>>) -> MOSA {
        MOSA {
            population_size: 50,
            mutation_rate: 0.2,
            selection_rate: 0.3,
            crossover_rate: 0.00001,
            generations: 100,
            rank_selection,
            branch_manager,
        }
    }

    pub fn generations(&mut self, generations: u64) -> &mut MOSA {
        self.generations = generations;
        self
    }

    pub fn population_size(&mut self, size: u64) -> &mut MOSA {
        self.population_size = size;
        self
    }

    pub fn run<C: Chromosome>(
        &mut self,
        mut source_file: SourceFile,
        initial_population: Vec<C>,
    ) -> Option<C> {
        let mut time = Time::new();

        let count = (self.generations + 1) * self.population_size;
        let mut pb = ProgressBar::new(count);
        pb.format("╢▌▌░╟");

        let mut current_generation = 0;

        time.start("population");
        let mut population = initial_population;
        time.end("population");

        time.start("source_file");
        //let mut source_file = SourceFile::new("/Users/tim/Documents/master-thesis/testify/src/examples/additions/src/main.rs");
        time.end("source_file");
        time.start("test_write");
        source_file.add_tests(&population, true);
        time.end("test_write");
        time.start("test_run");
        source_file.run_tests(&mut population);
        time.end("test_run");
        time.start("test_clear");
        source_file.clear_tests();
        time.end("test_clear");

        pb.add(self.population_size);
        let mut archive = &mut vec![];

        time.start("archive");
        self.update_archive(archive, &population);
        time.end("archive");

        while current_generation < self.generations {
            self.branch_manager
                .borrow_mut()
                .set_current_population(&population);

            time.start("population");
            let mut offspring = self.generate_offspring(&population)?;
            time.end("population");

            time.start("test_write");
            source_file.add_tests(&offspring, true);
            time.end("test_write");
            time.start("test_run");
            source_file.run_tests(&mut offspring);
            time.end("test_run");
            time.start("test_clear");
            source_file.clear_tests();
            time.end("test_clear");

            time.start("archive");
            self.update_archive(&mut archive, &offspring);
            time.end("archive");

            offspring.append(&mut population);

            // TODO there is a bug in sort, duplicate ids
            time.start("preference_sorting");
            let mut fronts =
                PreferenceSorter::sort(&offspring, self.branch_manager.borrow().branches());
            time.end("preference_sorting");

            time.start("fronts");
            for i in 0..fronts.len() {
                let front = fronts.get(&i).unwrap();
                let front = SVD::compute(front, self.branch_manager.borrow().branches())?;
                for t in &front {
                    population.push(t.clone());
                    if population.len() == self.population_size as usize {
                        break;
                    }
                }

                if population.len() == self.population_size as usize {
                    break;
                }
            }
            time.end("fronts");

            pb.add(self.population_size);
            current_generation += 1;
        }

        time.start("tests");
        source_file.add_tests(&population, true);
        time.end("tests");

        let mut tmp_file = File::create("fitness.log").unwrap();

        population.iter().for_each(|t| {
            let bm = self.branch_manager.borrow();
            let branches = bm.branches();
            let fitness = branches
                .iter()
                .map(|b| format!("b = {}, f = {}", b.id(), b.fitness(t)))
                .fold(String::new(), |acc, b| acc + &b.to_string() + ", ");
            let line = format!("Test {} => ({})\n", t.id(), fitness);
            tmp_file.write_all(&line.as_bytes());
        });

        let (uncovered_branches, coverage) = self.coverage();
        Some(TestSuite {
            uncovered_branches,
            coverage,
            tests: population,
        })
    }

    fn test_ids<C: Chromosome>(&self, tests: &[C]) -> HashSet<u64> {
        tests.iter().map(TestCase::id).collect()
    }

    fn crowding_distance_assignment<C: Chromosome>(&self, population: &mut [C]) {
        todo!()
    }

    fn coverage(&self) -> (Vec<Branch>, f64) {
        let bm = self.branch_manager.borrow();
        let uncovered_branches = bm.uncovered_branches();
        let coverage = 1.0 - uncovered_branches.len() as f64 / bm.branches().len() as f64;
        (uncovered_branches.to_vec(), coverage)
    }
}

struct Time {
    time: HashMap<String, u128>,
    timers: HashMap<String, Instant>,
}

impl Display for Time {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let values = self
            .time
            .iter()
            .map(|(k, v)| format!("{} => {}", k, *v as f64 / 1000000000f64))
            .fold(String::new(), |a, b| format!("{}, {}", a, b));

        f.write_fmt(format_args!("{}", values))
    }
}

impl Time {
    pub fn new() -> Self {
        Time {
            time: HashMap::new(),
            timers: HashMap::new(),
        }
    }

    pub fn start(&mut self, name: &str) {
        let now = Instant::now();
        self.timers.insert(name.to_owned(), now);
    }

    pub fn get(&self, name: &str) -> u128 {
        if let Some(v) = self.time.get(name) {
            *v
        } else {
            0
        }
    }

    pub fn end(&mut self, name: &str) {
        let elapsed = self
            .timers
            .get(name)
            .map(|t| t.elapsed().as_nanos())
            .unwrap();
        self.time
            .entry(name.to_owned())
            .and_modify(|e| *e += elapsed)
            .or_insert(elapsed);
    }
}

#[derive(Debug)]
pub struct PreferenceSorter {}

impl PreferenceSorter {
    pub fn sort<C: Chromosome>(population: &[C], objectives: &[Branch]) -> HashMap<usize, Vec<C>> {
        let mut fronts = HashMap::new();
        let mut front_0 = HashSet::new();
        let mut uncovered_branches = vec![];

        for objective in objectives {
            let mut min_dist = f64::MAX;
            let mut best_individual = None;

            for individual in population {
                let dist = individual.fitness(objective);
                if dist < min_dist {
                    best_individual = Some(individual);
                    min_dist = dist;
                }
            }

            if min_dist > 0.0 {
                uncovered_branches.push(objective);
                if let Some(individual) = best_individual {
                    front_0.insert(individual.clone());
                }
            }
        }

        fronts.insert(0, Vec::from_iter(front_0.to_owned()));
        let remaining_population: Vec<C> = population
            .iter()
            .filter(|&i| !front_0.contains(i))
            .map(|i| i.clone())
            .collect();
        if !remaining_population.is_empty() {
            let remaining_fronts = FNDS::sort(&remaining_population, objectives).unwrap();
            for i in 0..remaining_fronts.len() {
                fronts.insert(i + 1, remaining_fronts.get(&i).unwrap().to_owned());
            }
        }
        fronts
    }

    pub fn test_ids<C: Chromosome>(test_cases: &[C]) -> HashSet<u64> {
        test_cases.iter().map(C::id).collect()
    }

    pub fn ids<C: Chromosome>(fronts: &HashMap<usize, Vec<C>>) -> Vec<u64> {
        let mut ids = vec![];
        for (_, front) in fronts.iter() {
            for test_case in front {
                ids.push(test_case.id());
            }
        }

        ids
    }
}

struct Pareto {}

impl Pareto {
    pub fn dominates<C: Chromosome>(t1: &C, t2: &C, objectives: &[Branch]) -> bool {
        if objectives.iter().any(|m| t2.fitness(m) < t1.fitness(m)) {
            return false;
        }

        objectives.iter().any(|m| t1.fitness(m) < t2.fitness(m))
    }
}

struct FNDS {}

impl FNDS {
    pub fn sort<C: Chromosome>(
        population: &[C],
        objectives: &[Branch],
    ) -> Option<HashMap<usize, Vec<C>>> {
        let mut front: HashMap<usize, Vec<C>> = HashMap::new();
        let mut S = HashMap::new();
        let mut n = HashMap::new();
        (0..population.len()).for_each(|i| {
            let individual = population.get(i).unwrap();
            S.insert(individual, HashSet::new());
            n.insert(individual, 0);
        });

        for i in 0..population.len() {
            let p = population.get(i)?;

            for j in 0..population.len() {
                if i == j {
                    continue;
                }
                let q = population.get(j).unwrap();
                if Pareto::dominates(p, q, objectives) {
                    S.get_mut(p).unwrap().insert(q);
                } else if Pareto::dominates(q, p, objectives) {
                    let e = n.get_mut(p).unwrap();
                    *e += 1;
                }
            }

            if *n.get(p).unwrap() == 0 {
                front
                    .entry(0)
                    .and_modify(|e| e.push(p.clone()))
                    .or_insert(vec![p.clone()]);
            }
        }

        let mut i = 0;
        while !front.get(&i)?.is_empty() {
            let mut Q = HashSet::new();
            for p in front.get(&i).unwrap() {
                for &q in S.get(p).unwrap() {
                    let e = n.get_mut(q).unwrap();
                    *e -= 1;
                    if *e == 0 {
                        Q.insert(q);
                    }
                }
            }
            i += 1;

            let next_front = Q.iter().map(|&x| x.clone()).collect();
            front.insert(i, next_front);
        }

        Some(front)
    }
}

pub struct SVD {}

impl SVD {
    pub fn compute<C: Chromosome>(population: &[C], objectives: &[Branch]) -> Option<Vec<C>> {
        let mut distances = HashMap::new();
        for i in 0..population.len() {
            let a = population.get(i)?;
            distances.insert(a, 0u64);

            for j in 0..population.len() {
                if i == j {
                    continue;
                }

                let b = population.get(j)?;

                let v = SVD::svd(a, b, objectives);
                if *distances.get(a)? < v {
                    distances.insert(a, v);
                }
            }
        }

        let mut sorted_population = population.to_owned();
        sorted_population.sort_by(|a, b| distances.get(a).unwrap().cmp(distances.get(b).unwrap()));
        Some(sorted_population)
    }

    fn svd<C: Chromosome>(a: &C, b: &C, objectives: &[Branch]) -> u64 {
        let mut count = 0;
        for m in objectives {
            if b.fitness(m) < a.fitness(m) {
                count += 1;
            }
        }

        count
    }
}

pub struct Archive<C: Chromosome> {
    test_cases: Vec<C>,
    branch_manager: Rc<RefCell<BranchManager>>,
}

impl<C: Chromosome> Archive<C> {
    pub fn new(branch_manager: Rc<RefCell<BranchManager>>) -> Self {
        Archive {
            test_cases: vec![],
            branch_manager,
        }
    }

    pub fn update(&mut self, population: &[C]) {
        for branch in self.branch_manager.borrow().branches() {
            let mut best_test_case = None;
            let mut best_length = usize::MAX;
            if let Some(test_case) = self.test_that_covers(&branch) {
                best_length = test_case.size();
                best_test_case = Some(test_case);
            }

            for test_case in population {
                let score = branch.fitness(test_case);
                let length = test_case.size();
                if score == FitnessValue::Zero && length <= best_length {
                    if let Some(best_test_case) = best_test_case {
                        let i = self
                            .test_cases
                            .iter()
                            .position(|t| *t == best_test_case)
                            .unwrap();
                        self.archive[i] = test_case.clone();
                    } else {
                        self.archive.push(test_case.clone());
                    }
                    best_test_case = Some(test_case.clone());
                    best_length = length;
                }
            }
        }
    }

    fn test_that_covers(&self, branch: &Branch) -> Option<&C> {
        self.test_cases
            .iter()
            .filter(|&t| branch.fitness(t) == FitnessValue::Zero)
            .nth(0)
            .cloned()
    }
}

pub trait GenerateOffspring {
    fn generate<C: Chromosome>(&self, population: &[C]) -> Vec<C>;
}

pub struct OffspringGenerator<S: Selection> {
    selection: Rc<S>,
}

impl<S: Selection> OffspringGenerator<S> {
    pub fn new(selection: Rc<S>) -> Self {
        OffspringGenerator { selection }
    }
}

impl<S: Selection> GenerateOffspring for OffspringGenerator<S> {
    fn generate<C: Chromosome>(&self, population: &[C]) -> Vec<C> {
        let mut offspring = vec![];
        while offspring.len() < population.len() {
            let parent_1 = self.selection.apply(population);
            let parent_2 = self.selection.apply(population);

            let mut child_1: C;
            let mut child_2: C;

            if fastrand::f64() < self.crossover_rate {
                let (a, b) = parent_1.crossover(&parent_2);
                child_1 = a;
                child_2 = b;
            } else {
                child_1 = parent_1.clone();
                child_2 = parent_2.clone();
            }

            assert_ne!(parent_1.id(), child_1.id());
            assert_ne!(parent_2.id(), child_2.id());

            let mut mutated_child_1 = child_1.mutate();
            let mut mutated_child_2 = child_2.mutate();

            if population.len() - offspring.len() >= 2 {
                offspring.push(mutated_child_1);
                offspring.push(mutated_child_2);
            } else {
                offspring.push(if fastrand::f64() < 0.5 {
                    mutated_child_1
                } else {
                    mutated_child_2
                });
            }
        }

        offspring
    }
}

pub struct TestSuite<C: Chromosome> {
    pub coverage: f64,
    pub uncovered_branches: Vec<Branch>,
    pub tests: Vec<Chromosome>,
}
