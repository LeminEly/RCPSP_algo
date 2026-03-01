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

// --- FONCTIONS DE BASE ---

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
    let mut rng = rand::thread_rng();
    let idx = rng.gen_range(0..chrom.len() - 1);
    chrom.swap(idx, idx + 1);
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
    let mut adj = vec![vec![]; problem.n];
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

// --- LE SOLVER PRINCIPAL ---

pub fn solve_monstre(
    problem: &Problem,
    _pop_size: usize,
    epochs: usize,
    generations_per_epoch: usize
) -> (Vec<usize>, usize) {
    let num_islands = rayon::current_num_threads();
    let real_pop_size = 50; // Optimisé pour J120

    let mut islands: Vec<Vec<Vec<usize>>> = (0..num_islands)
        .map(|_| (0..real_pop_size).map(|_| random_topological_sort(problem)).collect())
        .collect();

    let mut global_best_chrom = Vec::new();
    let mut global_best_fit = usize::MAX;

    for _epoch in 0..epochs {
        let results: Vec<(Vec<Vec<usize>>, Vec<usize>, usize)> = islands.into_par_iter().map(|mut pop| {
            let mut rng = rand::thread_rng();
            let mut island_best_fit = usize::MAX;
            let mut island_best = Vec::new();

            for g in 0..generations_per_epoch {
                let fits: Vec<usize> = pop.iter().map(|c| fitness(problem, c)).collect();
                
                for (i, &f) in fits.iter().enumerate() {
                    if f < island_best_fit {
                        island_best_fit = f;
                        island_best = pop[i].clone();
                        // On n'optimise le champion que s'il est vraiment bon
                        if g % 10 == 0 {
                            island_best = double_justification(problem, &island_best);
                            island_best_fit = fitness(problem, &island_best);
                        }
                    }
                }

                let mut newpop = Vec::with_capacity(real_pop_size);
                newpop.push(island_best.clone());

                while newpop.len() < real_pop_size {
                    let p1 = select(&pop, &fits, 3);
                    let p2 = select(&pop, &fits, 3);
                    let mut child = crossover(problem, &p1, &p2);

                    if rng.gen_range(0.0..1.0) < 0.2 {
                        smart_mutation(problem, &mut child);
                        repair_topological_sort(problem, &mut child);
                    }

                    // LNS très rare pour débloquer
                    if g % 50 == 0 && rng.gen_range(0.0..1.0) < 0.05 {
                        child = lns(&child, 0.15);
                        repair_topological_sort(problem, &mut child);
                    }
                    
                    newpop.push(child);
                }
                pop = newpop;
            }
            (pop, island_best, island_best_fit)
        }).collect();

        islands = Vec::new();
        for (pop, b_chrom, b_fit) in results {
            islands.push(pop);
            if b_fit < global_best_fit {
                global_best_fit = b_fit;
                global_best_chrom = b_chrom;
           
            }
        }

        for island in islands.iter_mut() {
            island[0] = global_best_chrom.clone();
        }
    }
    (global_best_chrom, global_best_fit)
}

// --- SCHEDULER ET JUSTIFICATION ---

fn custom_ssgs(problem: &Problem, order: &[usize], precedences: &[Vec<usize>]) -> (Vec<usize>, usize) {
    let n = problem.n;
    let mut start = vec![0; n];
    let mut finish = vec![0; n];
    let max_horizon: usize = problem.durations.iter().sum();
    let num_res = problem.capacities.len();
    let mut usage = vec![0; max_horizon * num_res];

    for &j in order {
        let dur = problem.durations[j];
        let mut t = 0;
        for &p in &precedences[j] {
            if finish[p] > t { t = finish[p]; }
        }

        loop {
            let mut ok = true;
            for tau in t..(t + dur) {
                if tau >= max_horizon { break; }
                for k in 0..num_res {
                    if usage[tau * num_res + k] + problem.resources[j][k] > problem.capacities[k] {
                        ok = false; break;
                    }
                }
                if !ok { break; }
            }
            if ok {
                start[j] = t;
                finish[j] = t + dur;
                for tau in t..(t + dur) {
                    for k in 0..num_res { usage[tau * num_res + k] += problem.resources[j][k]; }
                }
                break;
            }
            t += 1;
        }
    }
    (start, *finish.iter().max().unwrap_or(&0))
}

pub fn double_justification(problem: &Problem, chrom: &[usize]) -> Vec<usize> {
    let (s_fwd, _) = custom_ssgs(problem, chrom, &problem.precedences);
    let mut rj_order = chrom.to_vec();
    rj_order.sort_by_key(|&j| std::cmp::Reverse(s_fwd[j] + problem.durations[j]));

    let mut rev_prec = vec![vec![]; problem.n];
    for i in 0..problem.n {
        for &j in &problem.precedences[i] { rev_prec[j].push(i); }
    }

    let (s_bwd, _) = custom_ssgs(problem, &rj_order, &rev_prec);
    let mut lj_order = rj_order.clone();
    lj_order.sort_by_key(|&j| std::cmp::Reverse(s_bwd[j] + problem.durations[j]));
    lj_order
}
