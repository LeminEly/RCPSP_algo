use std::collections::HashSet;
use rand::Rng;
use rand::prelude::SliceRandom;
use rayon::prelude::*;

#[derive(Clone)]
pub struct Problem {
    pub n: usize,
    pub durations: Vec<usize>,
    pub capacities: Vec<usize>,
    pub precedences: Vec<Vec<usize>>,
    pub resources: Vec<Vec<usize>>,
}

pub fn serial_schedule(problem: &Problem, order: &[usize]) -> (Vec<usize>, usize) {
    custom_ssgs(problem, order, &problem.precedences)
}

pub fn critical_activities(problem: &Problem, start: &[usize]) -> HashSet<usize> {
    let mut finish = vec![0; problem.n];
    let mut cmax = 0;
    for i in 0..problem.n {
        finish[i] = start[i] + problem.durations[i];
        if finish[i] > cmax { cmax = finish[i]; }
    }
    let mut crit = HashSet::new();
    for i in 0..problem.n {
        if finish[i] == cmax { crit.insert(i); }
    }
    crit
}

pub fn fitness(problem: &Problem, chrom: &[usize]) -> usize {
    serial_schedule(problem, chrom).1
}

pub fn select(pop: &[Vec<usize>], fits: &[usize], k: usize) -> Vec<usize> {
    let mut rng = rand::thread_rng();
    let mut best: Option<usize> = None;
    for _ in 0..k {
        let i = rng.gen_range(0..pop.len());
        match best {
            None => best = Some(i),
            Some(b) => {
                if fits[i] < fits[b] { best = Some(i); }
            }
        }
    }
    pop[best.unwrap()].clone()
}

pub fn crossover(problem: &Problem, p1: &[usize], p2: &[usize]) -> Vec<usize> {
    let (s, _) = serial_schedule(problem, p1);
    let crit = critical_activities(problem, &s);
    let mut child = Vec::with_capacity(problem.n);
    let mut in_child = vec![false; problem.n];
    for &g in p1 {
        if crit.contains(&g) {
            child.push(g);
            in_child[g] = true;
        }
    }
    for &g in p2 {
        if !in_child[g] {
            child.push(g);
            in_child[g] = true;
        }
    }
    child
}

pub fn smart_mutation(problem: &Problem, chrom: &mut [usize]) {
    let (start, _) = serial_schedule(problem, chrom);
    let mut worst = 0;
    let mut max_finish = 0;
    for j in 0..problem.n {
        let finish = start[j] + problem.durations[j];
        if finish > max_finish {
            max_finish = finish;
            worst = j;
        }
    }
    let pos = chrom.iter().position(|&x| x == worst).unwrap();
    let mut rng = rand::thread_rng();
    let newpos = rng.gen_range(0..chrom.len());
    chrom.swap(pos, newpos);
}

pub fn lns(chrom: &[usize], rate: f64) -> Vec<usize> {
    let nremove = (chrom.len() as f64 * rate) as usize;
    let mut rng = rand::thread_rng();
    let mut indices: Vec<usize> = (0..chrom.len()).collect();
    indices.shuffle(&mut rng);
    let remove_indices: HashSet<usize> = indices.into_iter().take(nremove).collect();
    let mut kept = Vec::new();
    let mut removed = Vec::new();
    for (i, &val) in chrom.iter().enumerate() {
        if remove_indices.contains(&i) { removed.push(val); }
        else { kept.push(val); }
    }
    removed.shuffle(&mut rng);
    kept.extend(removed);
    kept
}

fn repair_topological_sort(problem: &Problem, chrom: &mut [usize]) {
    let mut in_degree = vec![0; problem.n];
    let mut adj: Vec<Vec<usize>> = vec![vec![]; problem.n];
    for j in 0..problem.n {
        for &p in &problem.precedences[j] {
            adj[p].push(j);
            in_degree[j] += 1;
        }
    }
    let mut ready = Vec::new();
    for i in 0..problem.n {
        if in_degree[i] == 0 { ready.push(i); }
    }
    let mut repaired = Vec::with_capacity(problem.n);
    let mut chrom_pos = vec![0; problem.n];
    for (i, &task) in chrom.iter().enumerate() { chrom_pos[task] = i; }
    while !ready.is_empty() {
        ready.sort_by_key(|&task| std::cmp::Reverse(chrom_pos[task]));
        let u = ready.pop().unwrap();
        repaired.push(u);
        for &v in &adj[u] {
            in_degree[v] -= 1;
            if in_degree[v] == 0 { ready.push(v); }
        }
    }
    chrom.copy_from_slice(&repaired);
}

fn random_topological_sort(problem: &Problem) -> Vec<usize> {
    let mut chrom: Vec<usize> = (0..problem.n).collect();
    let mut rng = rand::thread_rng();
    chrom.shuffle(&mut rng);
    repair_topological_sort(problem, &mut chrom);
    chrom
}

pub fn solve_monstre(
    problem: &Problem,
    pop_per_island: usize,
    epochs: usize,
    generations_per_epoch: usize
) -> (Vec<usize>, usize) {
    let num_islands = rayon::current_num_threads();
    let mut islands: Vec<Vec<Vec<usize>>> = (0..num_islands)
        .map(|_| (0..pop_per_island).map(|_| random_topological_sort(problem)).collect())
        .collect();

    let mut global_best = Vec::new();
    let mut global_best_fit = usize::MAX;

    for _epoch in 0..epochs {
        let results: Vec<(Vec<Vec<usize>>, Vec<usize>, usize)> = islands.into_par_iter().map(|mut pop| {
            let mut rng = rand::thread_rng();
            let mut island_best_fit = usize::MAX;
            let mut island_best = Vec::new();
            let mutation_rate = 0.3;

            for _ in 0..generations_per_epoch {
                let fits: Vec<usize> = pop.iter().map(|c| fitness(problem, c)).collect();
                for (i, &f) in fits.iter().enumerate() {
                    if f < island_best_fit {
                        island_best_fit = f;
                        island_best = pop[i].clone();
                    }
                }

                let mut newpop = Vec::with_capacity(pop_per_island);
                newpop.push(island_best.clone());

                while newpop.len() < pop_per_island {
                    let p1 = select(&pop, &fits, 5);
                    let p2 = select(&pop, &fits, 5);
                    let mut child = crossover(problem, &p1, &p2);

                    if rng.gen_range(0.0..1.0) < mutation_rate {
                        smart_mutation(problem, &mut child);
                    }
                    if rng.gen_range(0.0..1.0) < 0.2 {
                        child = lns(&child, 0.25);
                    }
                    repair_topological_sort(problem, &mut child);
                    
                    if rng.gen_range(0.0..1.0) < 0.10 {
                        child = double_justification(problem, &child); 
                    }
                    newpop.push(child);
                }
                pop = newpop;
            }
            (pop, island_best, island_best_fit)
        }).collect();

        islands = Vec::new();
        let mut best_of_epoch_fit = usize::MAX;
        let mut best_of_epoch_chrom = Vec::new();

        for (pop, best_chrom, best_fit) in results.into_iter() {
            islands.push(pop);
            if best_fit < best_of_epoch_fit {
                best_of_epoch_fit = best_fit;
                best_of_epoch_chrom = best_chrom;
            }
        }

        if best_of_epoch_fit < global_best_fit {
            global_best_fit = best_of_epoch_fit;
            global_best = best_of_epoch_chrom.clone();
        }

        for island in islands.iter_mut() {
            island[0] = global_best.clone();
        }
    }
    (global_best, global_best_fit)
}

fn custom_ssgs(problem: &Problem, order: &[usize], precedences: &[Vec<usize>]) -> (Vec<usize>, usize) {
    let n = problem.n;
    let mut start = vec![0; n];
    let mut finish = vec![0; n];
    let max_horizon: usize = problem.durations.iter().sum();
    let num_res = problem.capacities.len();
    let mut usage = vec![0; max_horizon * num_res];

    for &j in order {
        let dur = problem.durations[j];
        let mut min_start_time = 0;
        for &p in &precedences[j] {
            if finish[p] > min_start_time {
                min_start_time = finish[p];
            }
        }

        if dur == 0 {
            start[j] = min_start_time;
            finish[j] = min_start_time;
            continue;
        }

        let mut t = min_start_time;
        loop {
            let mut resource_ok = true;
            for tau in t..(t + dur) {
                if tau >= max_horizon { break; }
                for k in 0..num_res {
                    if usage[tau * num_res + k] + problem.resources[j][k] > problem.capacities[k] {
                        resource_ok = false;
                        break;
                    }
                }
                if !resource_ok { break; }
            }

            if resource_ok {
                start[j] = t;
                finish[j] = t + dur;
                for tau in t..(t + dur) {
                    for k in 0..num_res {
                        usage[tau * num_res + k] += problem.resources[j][k];
                    }
                }
                break;
            } else {
                t += 1;
            }
        }
    }
    let makespan = *finish.iter().max().unwrap_or(&0);
    (start, makespan)
}

pub fn double_justification(problem: &Problem, chrom: &[usize]) -> Vec<usize> {
    let (start_fwd, _) = custom_ssgs(problem, chrom, &problem.precedences);
    let mut rj_order = chrom.to_vec();
    rj_order.sort_by_key(|&j| std::cmp::Reverse(start_fwd[j] + problem.durations[j]));

    let mut rev_prec: Vec<Vec<usize>> = vec![vec![]; problem.n];
    for i in 0..problem.n {
        for &j in &problem.precedences[i] {
            rev_prec[j].push(i);
        }
    }

    let (start_bwd, _) = custom_ssgs(problem, &rj_order, &rev_prec);
    let mut lj_order = rj_order.clone();
    lj_order.sort_by_key(|&j| std::cmp::Reverse(start_bwd[j] + problem.durations[j]));

    lj_order
}
