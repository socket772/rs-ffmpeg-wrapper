use clap::Parser;
use iced::font::Style;
use iced::theme::Svg;
use iced::widget::{image, row, svg, Button, Container, Image, TextInput};
use iced::{Font, Renderer, Sandbox, Settings, Theme};
use iced_aw::SelectionList;
use std::env::{self};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::process::Command;
use std::thread::{self};
use iced::widget::column;
use iced::Length::Fill;

static DEFAULT_INPUT:&str = "./input";
static DEFAULT_OUTPUT:&str = "./output";

#[derive(Parser, Debug)]
#[command(version, about, about = "Utility scritta in rust per convertire in massa file multimediali in mp3 usando il multi threading")]
struct Args {

	// Numero dei thread
	#[arg(short, long, default_value_t = num_cpus::get(), help="Definisci il numero di thread (file convertiti contemporaneamente)")]
	threadcount: usize,

	// cartella input
	#[arg(short, long, default_value = DEFAULT_INPUT, help="Definisci la cartella di input")]
	input: String,

	// cartella output
	#[arg(short, long, default_value = DEFAULT_OUTPUT, help="Definisci la cartella di output")]
	output: String,

	// specifica la posizione di ffmpeg
	#[arg(short, long, default_value = "ffmpeg", help="Definisci il percorso di ffmpeg (necessario se in Windows)")]
	program: String,

	// abilita la sovrascrittura dei file
	#[clap(short, long, help="Definisci se sovrascrivere i file già esistenti")]
	sovrascrivi: bool,

	// disabilita la gui
	#[clap(long, help="Esegui il programma in modalità headless")]
	nogui: bool,


	// Formato del file
	/*
		Questa sezione è solo per i test, non va considerata documentazione
		Funzionanti ->
		Non Testati ->
		Non Funzionanti -> 3gp (estensione accettata ma chiede opzioni particolari)
	*/
	#[arg(short, long, default_value = "mp3", help="Definisci l'estensione del file audio [mp3, m4a, flac, ogg, wav, aac, m4b, oga, opus, webm]")]
	formato: String
}

// Creo una struct per poter condividere le informazioni con i miei thread
struct Canzoni {
	// Vettore delle canzoni
	vettore_canzoni: Vec<String>,
	// Contatore di posizione
	posizione: usize,
	// Cartella di input
	input_folder: String,
	// Cartella di output
	output_folder: String,
	// Percorso di ffmpeg
	program: String,
	// Sovrascrittura abilitata
	sovrascrivi: bool,
	// Formato file
	formato: String
}

struct Gui {
	input_folder: String,
	output_folder: String,
	threads: usize,
	num_canzoni: usize,
}

enum Formats {
	"mp3",
	"m4a",
	"flac",
	"ogg",
	"wav",
	"aac",
	"m4b",
	"oga",
	"opus",
	"webm",
}

#[derive(Debug, Clone)]
enum GuiMessage {
	Start,
	InputFolder(String),
	OutputFolder(String),
	FileFormat(String),
	ThreadNumber(String),
}

impl Sandbox for Gui {
	type Message = GuiMessage;

	fn theme(&self) -> iced::Theme {
		iced::Theme::default()
	}

	fn style(&self) -> iced::theme::Application {
		iced::theme::Application::default()
	}

	fn scale_factor(&self) -> f64 {
		1.0
	}

	fn run(settings: iced::Settings<()>) -> Result<(), iced::Error>
	where
		Self: 'static + Sized,
	{
		<Self as iced::Application>::run(settings)
	}
	
	fn new() -> Self {
			Gui {
				input_folder: "./input".to_owned(),
				output_folder: "./output".to_owned(),
				threads: num_cpus::get().to_owned(),
				num_canzoni: 0.to_owned()
			}
		}
	
	fn title(&self) -> String {
			String::from("rs-ffmpeg-wrapper GUI")
		}
	
	fn update(&mut self, message: Self::Message) {
			match message {
				GuiMessage::Start => {
					println!("Start");
				},
				GuiMessage::InputFolder(value) => {
					self.input_folder = value;
				},
				GuiMessage::OutputFolder(_) => {
				},
				GuiMessage::FileFormat(_) => {
				},
				GuiMessage::ThreadNumber(_) => {
				},
			}
		}
	
	fn view(&self) -> iced::Element<'_, Self::Message> {

			let input_text: TextInput<GuiMessage> = TextInput::new("placeholder", self.input_folder.as_str()).on_input(GuiMessage::InputFolder);

			let output_text: TextInput<GuiMessage> = TextInput::new("placeholder", self.input_folder.as_str()).on_input(GuiMessage::InputFolder);

			let format_option: SelectionList<GuiMessage> = SelectionList::new(options, on_selected);
			
			let start_button = Button::new("Start").on_press(GuiMessage::Start);

			let col = column![input_text, output_text, start_button];

			return Container::new(col).center_x().width(iced::Length::Fill).into()
		}
}

fn main() {
	let _args: Vec<_> = env::args().collect();
	if _args.len() > 1 {
		println!("Running in headless mode");
		main_headless();
	}
	else {
		println!("Running in gui mode");
		let _ = Gui::run(Settings::default());
	}
}

fn main_headless() {
		// Instazio la variabile contenente gli argomenti
		let args = Args::parse();
		let threadcount:usize = args.threadcount;
		let input_folder_arg:String = args.input;
		let output_folder_arg:String = args.output;
	
		// Se il numero di thread è 0, termina il programma
		if threadcount == 0 {
			println!("Numero di thread invalido, inserire un numero maggiore di 0");
			return;
		}
	
		// Controllo se esiste la cartella di input
		if !Path::new(input_folder_arg.clone().as_str()).exists() {
			println!("La cartella di input non esiste");
			return;
		}
	
		// Creo la cartella di output nel caso non esiste, se esiste continua
		let output_folder_result = fs::create_dir_all(output_folder_arg.clone());
		if output_folder_result.is_ok() || Path::new(output_folder_arg.clone().as_str()).exists() {
			println!("Cartella output creata: {}", output_folder_arg)
		} else {
			println!("Cartella output non creata: {}", output_folder_arg);
			return;
		}
	
		// Creo una lista iteratore di stringhe con le canzoni
		let array_canzoni_temp = get_song_list(input_folder_arg.clone());
	
		// Se non ci sono canzoni, termina il programma
		if array_canzoni_temp.is_empty() {
			println!("Non ci sono canzoni nella cartella di input.");
			return;
		}
		else {
			println!("Ci sono canzoni nella cartella di input.");
		}
	
		// Controllo se l'estensione inserita è valtida
		match args.formato.as_str() {
			"mp3"|"m4a"|"flac"|"ogg"|"wav"|"aac"|"m4b"|"oga"|"opus"|"webm"=>println!("{}", args.formato),
			_=>{
				println!("Formato non supportato");
				return
			}
		}
	
		// Instanzio la struct
		// Nelle prossime versioni trasformerò tutto in una Lista
		let dati_condivisi:Canzoni = Canzoni {
			vettore_canzoni: array_canzoni_temp.clone(),
			posizione: 0,
			input_folder: input_folder_arg,
			output_folder: output_folder_arg,
			program: args.program,
			sovrascrivi: args.sovrascrivi,
			formato: args.formato
		};
	
		// Instanzio il lucchetto Mutex che usero per accedere ai dati condivisi
		let mutex_lock: Arc<Mutex<Canzoni>> = Arc::new(Mutex::new(dati_condivisi));
	
		// Inizio ciclo dei thread
		ciclo_threads(threadcount, mutex_lock, array_canzoni_temp.len());
}

fn ciclo_threads(threadcount: usize, mutex_lock: Arc<Mutex<Canzoni>>, numero_canzoni:usize) {
	// Vettore mutabile per gestire i thread, specialmente la parte di join
	let mut thread_vector: Vec<thread::JoinHandle<()>> = vec![];
	for _ in 0..threadcount {
		// Necessario, se no non posso utilizzarlo (capisci perchè)
		let mutex_lock = mutex_lock.clone();

		// Qui metto in nuovo thread nell'array, è come una lista.
		thread_vector.push(thread::spawn(move || {
			
			// Ciclo che passa tutte le canzoni se necessario, quasi garantito che esca prima
			for _ in 0..numero_canzoni {
				let result = thread_operation(&mutex_lock, numero_canzoni);
				if result == 1 {
					break;
				}
			}

			println!("Un thread ha finito")
		}));
	}

	// In questo ciclo faccio il join dei thread, altrimenti il programma padre termina prima dei thread
	for thread_element in thread_vector {
		let _ = thread_element.join();
	}
}

// questa funzione contiene il codice eseguito dai thread
fn thread_operation(mutex_lock: &Arc<Mutex<Canzoni>>, numero_canzoni:usize) -> i32 {
	// prendo il lucchetto mutex_lock
	let mut dati_condivisi = mutex_lock.lock().unwrap();

	// Copio le variabili che mi servono dalla struct
	let posizione_temp = dati_condivisi.posizione;
	let vettore_canzoni_temp = dati_condivisi.vettore_canzoni.clone();
	let input_folder = dati_condivisi.input_folder.clone();
	let output_folder = dati_condivisi.output_folder.clone();
	let program_temp = dati_condivisi.program.clone();
	let sovrascrivi_temp = dati_condivisi.sovrascrivi;
	let formato = dati_condivisi.formato.clone();

	
	// Controllo se ci sono altre canzoni da convertire
	if posizione_temp >= numero_canzoni {
		drop(dati_condivisi);
		return 1;
	}
	
	// Aumento di 1 il contatore globale
	dati_condivisi.posizione += 1;
	drop(dati_condivisi);
	
	// Estraggo il nome della canzione
	let nome_canzone = vettore_canzoni_temp[posizione_temp].clone();

	// Annuncio inizio canzone
	println!("Iniziata `{}` {}/{}", nome_canzone, posizione_temp+1, numero_canzoni);

	// Creo il percorso del file di input e output
	let canzone_input_path = format!("{}/{}", input_folder, nome_canzone);
	let canzone_output_path = format!("{}/{}.{}", output_folder, nome_canzone, formato);

	// Selezione se sovrascrivere o no i file
	let mut sovrascrivi_arg = "-n";
	if sovrascrivi_temp {
		sovrascrivi_arg = "-y";
	}

	/*
		{sovrascrivi_arg} -> usato per scegliere se sovrascrivere file dallo stesso nome o no
		-loglevel panic -> nascondi l'output
		-nostats -> togli le statistiche
		-vn -> rimuovi tutti i dati non relativi all'audio
	 */
	let argomenti = [sovrascrivi_arg, "-loglevel", "panic", "-nostats", "-i", canzone_input_path.as_str(), "-vn", canzone_output_path.as_str()];
	
	// Istanzio ed eseguo il comando di ffmpeg con le impostazioni scelte
	let command = Command::new(program_temp).args(argomenti).spawn().unwrap().wait();
	// Verifico risultato del comando
	if command.is_err() {
		// metti questa print fuori dalla funzione
		println!("Errore nel thread, esco");
		return -1;
	} else {
		println!("Finito `{}` {}/{}", nome_canzone, posizione_temp+1, numero_canzoni);
		return 0;
	}
}

fn get_song_list(input_folder_arg: String) -> Vec<String> {
	// Creo una lista iteratore di stringhe con le canzoni
	let lista_canzoni = fs::read_dir(input_folder_arg.clone()).unwrap();

	let mut array_canzoni_temp:Vec<String> = vec![];
	// La converto in un array, altrimenti non riesco a passarla ai thread
	for elemento in lista_canzoni {
		array_canzoni_temp.push(elemento.unwrap().file_name().into_string().unwrap());
	}

	return array_canzoni_temp;
}