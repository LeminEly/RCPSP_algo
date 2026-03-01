mod solver;
use solver::*;
use std::fs::{self, File};
use std::io::Write;
use std::collections::HashMap;
use std::time::Instant;
use clap::Parser;
use rayon::prelude::*;

#[derive(Parser)]
struct Args {
    #[arg(short='p', long)]
    dataset: String,
    
    #[arg(short='s', long)]
    solutions: String, 
}

fn parse_psplib(path: &str) -> Problem {
    let text = fs::read_to_string(path).expect("Impossible de lire le fichier");
    let mut durations = Vec::new();
    let mut resources = Vec::new();
    let mut precedences: Vec<Vec<usize>> = Vec::new();
    let mut capacities = Vec::new();

    enum Mode { None, Prec, Req, Cap }
    let mut mode = Mode::None;

    for line in text.lines() {
        let l = line.trim();
        if l.is_empty() || l.starts_with('*') { continue; }

        if l.starts_with("PRECEDENCE RELATIONS") { mode = Mode::Prec; continue; }
        if l.starts_with("REQUESTS/DURATIONS") { mode = Mode::Req; continue; }
        if l.starts_with("RESOURCEAVAILABILITIES") { mode = Mode::Cap; continue; }

        match mode {
            Mode::Prec => {
                let nums: Vec<&str> = l.split_whitespace().collect();
                if nums.len() < 3 || !nums[0].chars().all(|c| c.is_ascii_digit()) { continue; }
                let job: usize = nums[0].parse().unwrap();
                let nsucc: usize = nums[2].parse().unwrap();
                while precedences.len() < job { precedences.push(vec![]); }
                for i in 0..nsucc {
                    if let Some(succ_str) = nums.get(3 + i) {
                        let succ: usize = succ_str.parse().unwrap();
                        while precedences.len() < succ { precedences.push(vec![]); }
                        precedences[succ - 1].push(job - 1);
                    }
                }
            }
            Mode::Req => {
                let nums: Vec<&str> = l.split_whitespace().collect();
                if nums.len() < 4 || !nums[0].chars().all(|c| c.is_ascii_digit()) { continue; }
                durations.push(nums[2].parse().unwrap());
                let mut r = Vec::new();
                for v in &nums[3..] {
                    r.push(v.parse().unwrap());
                }
                resources.push(r);
            }
            Mode::Cap => {
                let nums: Vec<&str> = l.split_whitespace().collect();
                let caps: Vec<usize> = nums.iter().filter_map(|x| x.parse::<usize>().ok()).collect();
                if !caps.is_empty() {
                    capacities = caps;
                }
            }
            Mode::None => {}
        }
    }

    Problem { n: durations.len(), durations, capacities, precedences, resources }
}

fn parse_solutions(path: &str) -> HashMap<String, usize> {
    let mut records = HashMap::new();
    if let Ok(text) = fs::read_to_string(path) {
        for line in text.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                if let (Ok(g), Ok(i), Ok(makespan)) = (parts[0].parse::<usize>(), parts[1].parse::<usize>(), parts[2].parse::<usize>()) {
                    
                    records.insert(format!("{}_{}", g, i), makespan);
                }
            }
        }
    }
    records
}

fn main() {
    let args = Args::parse();
    
    let old_records = parse_solutions(&args.solutions);

    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(&args.dataset) {
        for entry in entries {
            let path = entry.unwrap().path();
            if path.extension().and_then(|s| s.to_str()) == Some("sm") {
                files.push(path);
            }
        }
    }

    files.sort();
    println!("Instances trouvées: {}", files.len());
    
    let results: Vec<(String, usize, f64, usize)> = files.into_par_iter().map(|file| {
        let name = file.file_name().unwrap().to_string_lossy().replace(".sm", "");
        
        
        let key = name.replace("j120", "").replace("j60", "").replace("j30", "").replace("j90", "");

        let problem = parse_psplib(file.to_str().unwrap());
        
        let start_time = Instant::now();
      
        let (_, best_score) = solve_monstre(&problem, 20, 2, 20);
        let time_sec = start_time.elapsed().as_secs_f64();

      
        let old_val = *old_records.get(&key).unwrap_or(&usize::MAX);
        
        (name, best_score, time_sec, old_val)
    }).collect();

    let mut file_out = File::create("resultats_finaux.txt").expect("Impossible de créer le fichier");
    writeln!(file_out, "Instance | Type Solution | Valeur | Technique de résolution | Temps d'exécution | Caracteristique machine | Ancienne Valeur").unwrap();

    let mut records_battus = 0;
    
    for (name, best_score, time_sec, old_val) in results {
        let old_str = if old_val == usize::MAX { "Inconnu".to_string() } else { old_val.to_string() };
        let line = format!("{} | HEUR | {} | GA+Îlots+DoubleJustif | {:.2}s | CPU Multi-core | {}", 
            name, best_score, time_sec, old_str);
        
        writeln!(file_out, "{}", line).unwrap();
        
        if best_score <= old_val && old_val != usize::MAX {
            println!("🎯 RECORD BATTU OU EGALÉ : {}", line);
            records_battus += 1;
        }
    }

    println!("==================================================");
    println!("Terminé ! Ouvre le fichier 'resultats_finaux.txt' dans ton dossier.");
    println!("Nombre de records battus ou égalés : {}", records_battus);
}
