use crate::chromosome::{ChromosomeGenerator, Chromosome, TestCaseGenerator, TestCase};
use std::rc::Rc;
use std::collections::{VecDeque, HashMap, HashSet};
use std::ops::Deref;
use crate::instr::data::Branch;
use std::cmp::Ordering;
use std::option::Option::Some;
use crate::operators::RankSelection;
use std::cell::RefCell;
use std::iter::FromIterator;
use crate::io::SourceFile;
use crate::generators::TestIdGenerator;
use std::collections::hash_map::DefaultHasher;

#[derive(Debug)]
pub struct MOSA {
    population_size: u64,
    mutation_rate: f64,
    selection_rate: f64,
    crossover_rate: f64,
    chromosome_generator: TestCaseGenerator,
    objectives: Vec<Branch>,
    generations: u64,
    rank_selection: RankSelection,
    test_id_generator: Rc<RefCell<TestIdGenerator>>,
}

impl MOSA {
    pub fn new(generator: TestCaseGenerator, rank_selection: RankSelection, branches: Vec<Branch>, test_id_generator: Rc<RefCell<TestIdGenerator>>) -> MOSA {
        MOSA {
            population_size: 50,
            mutation_rate: 0.2,
            selection_rate: 0.3,
            crossover_rate: 0.7,
            chromosome_generator: generator,
            objectives: branches,
            generations: 100,
            rank_selection,
            test_id_generator,
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

    pub fn run(&mut self) -> Option<(Vec<u64>, f64)> {
        // TODO may be this should be a set
        let mut current_generation = 0;
        let mut population = self.generate_random_population();

        /*let mut source_lut: HashMap<&SourceFile, Vec<TestCase>> = HashMap::new();
        for test_case in &population {
            let source_file = test_case.objective().source_file();
            source_lut.entry(source_file)
                .and_modify(|e| e.push(test_case.clone()))
                .or_insert(vec![test_case.clone()]);
        }*/

        let mut source_file = SourceFile::new("/Users/tim/Documents/master-thesis/testify/src/examples/additions/src/main.rs");
        source_file.add_tests(&population);
        source_file.run_tests(&mut population);
        source_file.clear_tests();
        let mut archive = &mut vec![];

        self.update_archive(archive, &population);
        while current_generation < self.generations {
            let mut offspring = self.generate_offspring(&population)?;
            source_file.add_tests(&offspring);
            source_file.run_tests(&mut offspring);
            source_file.clear_tests();


            self.update_archive(&mut archive, &offspring);
            offspring.append(&mut population);

            // TODO there is a bug in sort, duplicate ids
            let mut fronts = PreferenceSorter::sort(&offspring, &self.objectives);

            for i in 0..fronts.len() {
                let front = fronts.get(&i).unwrap();
                let front = SVD::compute(front, &self.objectives)?;
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

            current_generation += 1;
        }

        source_file.add_tests(&population);
        Some(self.coverage(&population))
    }

    fn test_ids(&self, tests: &[TestCase]) -> HashSet<u64> {
        tests.iter().map(TestCase::id).collect()
    }

    fn test_that_covers(&self, archive: &[TestCase], branch: &Branch) -> Option<TestCase> {
        archive.iter().filter(|&t| branch.fitness(t) == 0.0).nth(0).cloned()
    }

    fn update_archive(&self, archive: &mut Vec<TestCase>, population: &[TestCase]) {
        for branch in &self.objectives {
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

    fn coverage(&self, population: &[TestCase]) -> (Vec<u64>, f64) {
        let uncovered_branches = self.uncovered_branches(population);
        let uncovered_branch_ids: Vec<u64> = uncovered_branches.iter()
            .map(|b| b.id())
            .copied()
            .collect();
        let coverage = 1.0 - uncovered_branches.len() as f64 / self.objectives.len() as f64;
        (uncovered_branch_ids, coverage)
    }

    fn uncovered_branches(&self, population: &[TestCase]) -> Vec<Branch> {
        let mut uncovered_branches = vec![];
        for branch in &self.objectives {
            let mut covered = false;
            for individual in population {
                if individual.fitness(branch) == 0.0 {
                    covered = true;
                    break;
                }
            }

            if !covered {
                uncovered_branches.push(branch.clone());
            }
        }

        uncovered_branches
    }


    fn generate_offspring(&self, population: &[TestCase]) -> Option<Vec<TestCase>> {
        let mut offspring = vec![];
        let uncovered_objectives = self.uncovered_branches(population);
        while offspring.len() < population.len() {
            let parent_1 = self.rank_selection.select(population)?;
            let parent_2 = self.rank_selection.select(population)?;

            let child_1: TestCase;
            let child_2: TestCase;

            if fastrand::f64() < self.crossover_rate {
                let (a, b) = parent_1.crossover(&parent_2);
                child_1 = a;
                child_2 = b;
            } else {
                child_1 = parent_1.clone();
                child_2 = parent_2.clone();
            }

            let mut mutated_child_1 = child_1.mutate();
            let mut mutated_child_2 = child_2.mutate();

            mutated_child_1.set_id(self.test_id_generator.borrow_mut().next_id());
            mutated_child_2.set_id(self.test_id_generator.borrow_mut().next_id());

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
    /*pub fn sort(population: &[TestCase], objectives: &[Branch]) -> HashMap<usize, Vec<TestCase>> {
        let mut fronts: HashMap<usize, Vec<TestCase>> = HashMap::new();
        let mut S = HashMap::new();
        let mut n = HashMap::new();

        for p in population {
            S.insert(p, HashSet::new());
            n.insert(p, 0u32);
            for q in population {
                if Pareto::dominates(p, q, objectives) {
                    if let Some(dominated_tests) = S.get_mut(p) {
                        dominated_tests.insert(q.clone());
                    }
                } else if Pareto::dominates(q, p, objectives) {
                    n.entry(p).and_modify(|e| *e += 1).or_insert(1);
                }
            }

            if let Some(0) = n.get(p) {
                // TODO p_rank = 1
                fronts.entry(0)
                    .and_modify(|e| e.push(p.clone()))
                    .or_insert(vec![p.clone()]);
            }
        }

        let mut i = 0;
        while let Some(front) = fronts.get(&i) {
            if front.is_empty() { break; }
            let mut Q = HashSet::new();
            for p in front {
                if let Some(dominated_tests) = S.get(p) {
                    for q in dominated_tests {
                        if let Some(e) = n.get_mut(q) {
                            *e -= 1;
                            if *e == 0 {
                                Q.insert(q.clone());
                            }
                        } else {
                            panic!("THis should never happen");
                        }
                    }
                }
            }
            i += 1;
            fronts.insert(i, Vec::from_iter(Q));
        }

        fronts.remove(&(fronts.len() - 1));

        fronts
    }*/

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