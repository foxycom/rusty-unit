use crate::chromosome::{ChromosomeGenerator, Chromosome, TestCaseGenerator, TestCase};
use std::rc::Rc;
use std::collections::{VecDeque, HashMap, HashSet};
use std::ops::Deref;
use std::cmp::Ordering;
use std::option::Option::Some;
use crate::operators::RankSelection;
use std::cell::RefCell;
use std::iter::FromIterator;
use crate::source::{SourceFile, BranchManager, Branch};
use crate::generators::TestIdGenerator;
use std::collections::hash_map::DefaultHasher;
use pbr::ProgressBar;
use std::time::Instant;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Write;

#[derive(Debug)]
pub struct MOSA {
    population_size: u64,
    mutation_rate: f64,
    selection_rate: f64,
    crossover_rate: f64,
    chromosome_generator: TestCaseGenerator,
    branch_manager: Rc<RefCell<BranchManager>>,
    generations: u64,
    rank_selection: RankSelection,
    test_id_generator: Rc<RefCell<TestIdGenerator>>,
}

impl MOSA {
    pub fn new(generator: TestCaseGenerator, rank_selection: RankSelection, branch_manager: Rc<RefCell<BranchManager>>, test_id_generator: Rc<RefCell<TestIdGenerator>>) -> MOSA {
        MOSA {
            population_size: 50,
            mutation_rate: 0.2,
            selection_rate: 0.3,
            crossover_rate: 0.00001,
            chromosome_generator: generator,
            generations: 100,
            rank_selection,
            test_id_generator,
            branch_manager,
        }
    }

    pub fn chromosome_generator(&mut self, generator: TestCaseGenerator) -> &mut MOSA {
        self.chromosome_generator = generator;
        self
    }

    pub fn generations(&mut self, generations: u64) -> &mut MOSA {
        self.generations = generations;
        self
    }

    pub fn population_size(&mut self, size: u64) -> &mut MOSA {
        self.population_size = size;
        self
    }

    pub fn run(&mut self, mut source_file: SourceFile) -> Option<GaResult> {
        let mut time = Time::new();

        let count = (self.generations + 1) * self.population_size;
        let mut pb = ProgressBar::new(count);
        pb.format("╢▌▌░╟");

        let mut current_generation = 0;

        time.start("population");
        let mut population = self.generate_random_population();
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
            self.branch_manager.borrow_mut().set_current_population(&population);

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
            let mut fronts = PreferenceSorter::sort(&offspring, self.branch_manager.borrow().branches());
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
            let fitness = branches.iter()
                .map(|b| format!("b = {}, f = {}", b.id(), b.fitness(t)))
                .fold(String::new(), |acc, b| acc + &b.to_string() + ", ");
            let line = format!("Test {} => ({})\n", t.id(), fitness);
            tmp_file.write_all(&line.as_bytes());
        });

        //println!("\n{}", time);

        let (uncovered_branches, coverage) = self.coverage();
        Some(GaResult{
            uncovered_branches,
            coverage,
            tests: population
        })
    }

    fn test_ids(&self, tests: &[TestCase]) -> HashSet<u64> {
        tests.iter().map(TestCase::id).collect()
    }

    fn test_that_covers(&self, archive: &[TestCase], branch: &Branch) -> Option<TestCase> {
        archive.iter().filter(|&t| branch.fitness(t) == 0.0).nth(0).cloned()
    }

    fn update_archive(&self, archive: &mut Vec<TestCase>, population: &[TestCase]) {
        for branch in self.branch_manager.borrow().branches() {
            let mut best_test_case = None;
            let mut best_length = usize::MAX;
            if let Some(test_case) = self.test_that_covers(archive, &branch) {
                best_length = test_case.size();
                best_test_case = Some(test_case);
            }

            for test_case in population {
                let score = branch.fitness(test_case);
                let length = test_case.size();
                if score == 0.0 && length <= best_length {
                    if let Some(best_test_case) = best_test_case {
                        let i = archive.iter().position(|t| *t == best_test_case).unwrap();
                        archive[i] = test_case.clone();
                    } else {
                        archive.push(test_case.clone());
                    }
                    best_test_case = Some(test_case.clone());
                    best_length = length;
                }
            }
        }
    }

    fn crowding_distance_assignment(&self, population: &mut [TestCase]) {
        todo!()
    }

    fn coverage(&self) -> (Vec<Branch>, f64) {
        let bm = self.branch_manager.borrow();
        let uncovered_branches = bm.uncovered_branches();
        let coverage = 1.0 - uncovered_branches.len() as f64 / bm.branches().len() as f64;
        (uncovered_branches.to_vec(), coverage)
    }

    fn generate_offspring(&self, population: &[TestCase]) -> Option<Vec<TestCase>> {
        let mut offspring = vec![];
        while offspring.len() < population.len() {
            let parent_1 = self.rank_selection.select(population)?;
            let parent_2 = self.rank_selection.select(population)?;

            let mut child_1: TestCase;
            let mut child_2: TestCase;

            if fastrand::f64() < self.crossover_rate {
                let (a, b) = parent_1.crossover(&parent_2);
                child_1 = a;
                child_2 = b;
            } else {
                child_1 = parent_1.clone();
                child_2 = parent_2.clone();
            }

            child_1.set_id(self.test_id_generator.borrow_mut().next_id());
            child_2.set_id(self.test_id_generator.borrow_mut().next_id());
            let mut mutated_child_1 = child_1.mutate();
            let mut mutated_child_2 = child_2.mutate();

            /*mutated_child_1.set_id(self.test_id_generator.borrow_mut().next_id());
            mutated_child_2.set_id(self.test_id_generator.borrow_mut().next_id());*/

            if population.len() - offspring.len() >= 2 {
                offspring.push(mutated_child_1);
                offspring.push(mutated_child_2);
            } else {
                offspring.push(if fastrand::f64() < 0.5 { mutated_child_1 } else { mutated_child_2 });
            }
        }

        Some(offspring)
    }

    fn generate_random_population(&mut self) -> Vec<TestCase> {
        let mut population = Vec::new();
        for _ in 0..self.population_size {
            population.push(self.chromosome_generator.generate());
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

struct Time {
    time: HashMap<String, u128>,
    timers: HashMap<String, Instant>,
}

impl Display for Time {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let values = self.time.iter()
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
        let elapsed = self.timers
            .get(name)
            .map(|t| t.elapsed().as_nanos())
            .unwrap();
        self.time.entry(name.to_owned()).and_modify(|e| *e += elapsed).or_insert(elapsed);
    }
}


#[derive(Debug)]
pub struct PreferenceSorter {}

impl PreferenceSorter {
    pub fn sort(population: &[TestCase], objectives: &[Branch]) -> HashMap<usize, Vec<TestCase>> {
        let mut fronts = HashMap::new();
        let mut front_0 = HashSet::new();
        let mut uncovered_branches = vec![];


        for objective in objectives {
            let mut min_dist = f64::MAX;
            let mut best_individual: Option<&TestCase> = None;

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
        let remaining_population: Vec<TestCase> = population.iter()
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

    pub fn test_ids(tests: &[TestCase]) -> HashSet<u64> {
        tests.iter().map(TestCase::id).collect()
    }

    pub fn ids(fronts: &HashMap<usize, Vec<TestCase>>) -> Vec<u64> {
        let mut ids = vec![];
        for (i, front) in fronts.iter() {
            for test_case in front {
                ids.push(test_case.id());
            }
        }

        ids
    }
}

struct Pareto {}

impl Pareto {
    pub fn dominates(t1: &TestCase, t2: &TestCase, objectives: &[Branch]) -> bool {
        if objectives.iter().any(|m| t2.fitness(m) < t1.fitness(m)) {
            return false;
        }

        objectives.iter().any(|m| t1.fitness(m) < t2.fitness(m))
    }
}

struct FNDS {}

impl FNDS {
    pub fn sort(population: &[TestCase], objectives: &[Branch]) -> Option<HashMap<usize, Vec<TestCase>>> {
        let mut front: HashMap<usize, Vec<TestCase>> = HashMap::new();
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
                if i == j { continue; }
                let q = population.get(j).unwrap();
                if Pareto::dominates(p, q, objectives) {
                    S.get_mut(p).unwrap().insert(q);
                } else if Pareto::dominates(q, p, objectives) {
                    let e = n.get_mut(p).unwrap();
                    *e += 1;
                }
            }

            if *n.get(p).unwrap() == 0 {
                front.entry(0)
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
    pub fn compute(population: &[TestCase], objectives: &[Branch]) -> Option<Vec<TestCase>> {
        let mut distances = HashMap::new();
        for i in 0..population.len() {
            let a = population.get(i)?;
            distances.insert(a, 0u64);

            for j in 0..population.len() {
                if i == j { continue; }

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

    fn svd(a: &TestCase, b: &TestCase, objectives: &[Branch]) -> u64 {
        let mut count = 0;
        for m in objectives {
            if b.fitness(m) < a.fitness(m) {
                count += 1;
            }
        }

        count
    }
}

pub struct GaResult {
    pub coverage: f64,
    pub uncovered_branches: Vec<Branch>,
    pub tests: Vec<TestCase>
}