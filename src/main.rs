use clap::Parser;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::process::Command;
use std::thread::{self};

static DEFAULT_INPUT:&str = "./input";
static DEFAULT_OUTPUT:&str = "./output";

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
	// Numero dei thread
	#[arg(short, long, default_value_t = num_cpus::get())]
	threadcount: usize,

	// cartella input
	#[arg(short, long, default_value = DEFAULT_INPUT)]
	input: String,

	// cartella output
	#[arg(short, long, default_value = DEFAULT_OUTPUT)]
	output: String,

	// specifica la posizione di ffmpeg
	#[arg(short, long, default_value = "ffmpeg")]
	program: String,
}

// Creo una struct per poter condividere le informazioni con i miei thread
struct Canzoni {
	// Vettore delle canzoni
	vettore_canzoni: Vec<String>,
	// Numero di canzoni nella lista
	numero_canzoni: usize,
	// Contatore di posizione
	posizione: usize,
	// Cartella di input
	input_folder: String,
	// Cartella di output
	output_folder: String,
	// Percorso di ffmpeg
	program: String
}

fn main() {

	// Instazio la variabile contenente gli argomenti
	let args = Args::parse();
	let threadcount:usize = args.threadcount;
	let input_folder_arg:String = args.input;
	let output_folder_arg:String = args.output;

	// Controllo se esiste la cartella di input
	if !Path::new(input_folder_arg.clone().as_str()).exists() {
		panic!("La cartella di input non esiste")
	}

	// Creo la cartella di output nel caso non esiste, se esiste continua
	let output_folder_result = fs::create_dir_all(output_folder_arg.clone());
	if output_folder_result.is_ok() || Path::new(output_folder_arg.clone().as_str()).exists() {
		println!("Cartella output creata: {}", output_folder_arg)
	} else {
		panic!("Cartella output non creata: {}", output_folder_arg)
	}

	// Creo una lista iteratore di stringhe con le canzoni
	let lista_canzoni = fs::read_dir(input_folder_arg.clone()).unwrap();
	
	let mut array_canzoni_temp:Vec<String> = vec!["".to_string()];
	// La converto in un array, altrimenti non riesco a passarla ai thread
	for elemento in lista_canzoni {
		array_canzoni_temp.push(elemento.unwrap().file_name().into_string().unwrap());
	}

	println!("DEBUG: \n {:?}", array_canzoni_temp);

	// Perndo il numero di canzoni nella cartella
	let numero_canzoni = array_canzoni_temp.len();

	if numero_canzoni == 0 {
		println!("Non ci sono canzoni nella cartella di input.");
		return;
	}

	// Recupero percorso di ffmpeg
	let program = args.program;
	
	// Instanzio la struct
	// Nelle prossime versioni trasformerò tutto in una Lista
	let dati_condivisi:Canzoni = Canzoni {
		vettore_canzoni: array_canzoni_temp,
		numero_canzoni: numero_canzoni,
		posizione: 0,
		input_folder: input_folder_arg,
		output_folder: output_folder_arg,
		program: program
	};

	// Instanzio il lucchetto Mutex che usero per accedere ai dati condivisi
	let mutex_lock = Arc::new(Mutex::new(dati_condivisi));

	// Vettore mutabile per gestire i thread, specialmente la parte di join
	let mut thread_vector: Vec<thread::JoinHandle<()>> = vec![];

	for _ in 0..threadcount {
		// Necessario, se no non posso utilizzarlo (capisci perchè)
		let mutex_lock = mutex_lock.clone();

		// Qui metto in nuovo thread nell'array, è come una lista.
		thread_vector.push(thread::spawn(move || {
			
			// Ciclo che passa tutte le canzoni se necessario, quasi garantito che esca prima
			for _ in 0..numero_canzoni {
				// Instanzio il contatore protetto dal mutex_lock e chiudo il lucchetto
				let mut dati_condivisi = mutex_lock.lock().unwrap();

				// Copio le variabili che mi servono dalla struct
				let posizione_temp = dati_condivisi.posizione;
				let numero_canzoni_temp = dati_condivisi.numero_canzoni;
				let vettore_canzoni_temp = dati_condivisi.vettore_canzoni.clone();
				let input_folder = dati_condivisi.input_folder.clone();
				let output_folder = dati_condivisi.output_folder.clone();
				let program_temp = dati_condivisi.program.clone();

				
				// Controllo se ci sono altre canzoni da convertire
				if posizione_temp >= numero_canzoni_temp {
					drop(dati_condivisi);
					break;
				}
				
				// Aumento di 1 il contatore globale
				dati_condivisi.posizione = dati_condivisi.posizione + 1;
				drop(dati_condivisi);
				
				println!("Iniziata {}/{}", posizione_temp+1, numero_canzoni_temp);
				
				// Estraggo il nome della canzione
				let nome_canzone = vettore_canzoni_temp[posizione_temp].clone();

				// Creo il percorso del file di input e output
				let canzone_input_path = format!("{}/{}", input_folder, nome_canzone);
				let canzone_output_path = format!("{}/{}.mp3", output_folder, nome_canzone);

				let argomenti = ["-loglevel", "panic", "-nostats", "-i", canzone_input_path.as_str(),"-c:v", "copy", "-c:a", "libmp3lame", "-q:a", "4", "-threads", "4", canzone_output_path.as_str()];
				
				let command_result = Command::new(program_temp).args(argomenti).spawn();
				if command_result.is_err(){
					println!("Errore nel thread, esco");
					break;
				}
				
				println!("Finito {}/{}",posizione_temp+1, numero_canzoni_temp);
			}

			println!("Un thread ha finito")
		}));
	}

	// In questo ciclo faccio il join dei thread, altrimenti il programma padre termina prima dei thread
	for thread_element in thread_vector {
		let _ = thread_element.join();
	}
	return
}