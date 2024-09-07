#![allow(clippy::needless_return)]

use clap::Parser;
use iced::widget::{column, Checkbox};
use iced::widget::{Button, Column, Container, TextInput};
use iced::{Length, Padding, Sandbox, Settings};
use iced_aw::NumberInput;
use iced_aw::SelectionList;
use std::env::{self};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread::{self};

// Variabili globali
const DEFAULT_INPUT: &str = "./input";
const DEFAULT_OUTPUT: &str = "./output";
const FORMATS: [&str; 10] = [
    "mp3", "m4a", "flac", "ogg", "wav", "aac", "m4b", "oga", "opus", "webm",
];

#[derive(Parser, Debug)]
#[command(
    version,
    about,
    about = "Utility scritta in rust per convertire in massa file multimediali in mp3 usando il multi threading"
)]
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
    #[arg(
        short,
        long,
        default_value = "ffmpeg",
        help = "Definisci il percorso di ffmpeg (necessario se in Windows)"
    )]
    program: String,

    // abilita la sovrascrittura dei file
    #[clap(short, long, help = "Definisci se sovrascrivere i file già esistenti")]
    sovrascrivi: bool,

    // disabilita la gui
    #[clap(long, help = "Esegui il programma in modalità headless")]
    nogui: bool,

    // Formato del file
    /*
        Questa sezione è solo per i test, non va considerata documentazione
        Funzionanti ->
        Non Testati ->
        Non Funzionanti -> 3gp (estensione accettata ma chiede opzioni particolari)
    */
    #[arg(
        short,
        long,
        default_value = "mp3",
        help = "Definisci l'estensione del file audio [mp3, m4a, flac, ogg, wav, aac, m4b, oga, opus, webm]"
    )]
    formato: String,
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
    formato: String,
}

#[derive(Debug, Clone)]
struct Gui {
    input_folder: String,
    output_folder: String,
    ffmpeg_path: String,
    threads: usize,
    formats: Vec<String>,
    format_index: usize,
    format_selected: String,
    overwrite: bool,
}

#[derive(Debug, Clone)]
enum GuiMessage {
    Start,
    InputFolder(String),
    OutputFolder(String),
    FfmpegPath(String),
    Format(usize, String),
    ThreadNumber(usize),
    Overwrite(bool),
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
        let mut formats_final: Vec<String> = vec![];
        for formato in FORMATS {
            formats_final.push(formato.to_string());
        }
        Gui {
            input_folder: "./input".to_string(),
            output_folder: "./output".to_string(),
            ffmpeg_path: "ffmpeg".to_string(),
            threads: num_cpus::get(),
            formats: formats_final,
            format_index: 0,
            format_selected: String::from("mp3"),
            overwrite: false,
        }
    }

    fn title(&self) -> String {
        String::from("rs-ffmpeg-wrapper GUI")
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            GuiMessage::Start => {
                println!("{:?}", self);
                main_gui(self.clone());
            }
            GuiMessage::InputFolder(value) => {
                self.input_folder = value;
            }
            GuiMessage::OutputFolder(value) => {
                self.output_folder = value;
            }
            GuiMessage::Format(index, format) => {
                self.format_index = index;
                self.format_selected = format;
            }
            GuiMessage::ThreadNumber(number) => {
                self.threads = number;
            }
            GuiMessage::Overwrite(overwrite) => {
                self.overwrite = overwrite;
            }
            GuiMessage::FfmpegPath(path) => {
                self.ffmpeg_path = path;
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let input_text: TextInput<GuiMessage> =
            TextInput::new("input folder here", self.input_folder.as_str())
                .on_input(GuiMessage::InputFolder)
                .padding(10);

        let output_text: TextInput<GuiMessage> =
            TextInput::new("output folder here", self.output_folder.as_str())
                .on_input(GuiMessage::OutputFolder)
                .padding(10);

        let ffmpeg_path_text: TextInput<GuiMessage> =
            TextInput::new("path of ffmpeg executable here", self.ffmpeg_path.as_str())
                .on_input(GuiMessage::FfmpegPath)
                .padding(10);

        let format_option: SelectionList<String, GuiMessage> =
            SelectionList::new(&self.formats, GuiMessage::Format).height(Length::Fixed(100.0));

        let thread_number: NumberInput<usize, GuiMessage> =
            NumberInput::new(self.threads, 4096, GuiMessage::ThreadNumber)
                .padding(10.0)
                .step(1)
                .bounds((1, 4096));

        let sovrascrivi_checkbox: Checkbox<GuiMessage> = Checkbox::new("Sovrascrivi", false)
            .on_toggle(GuiMessage::Overwrite)
            .spacing(5);

        let start_button: Button<GuiMessage> =
            Button::new("Start").on_press(GuiMessage::Start).padding(10);

        let col: Column<GuiMessage> = column![
            input_text,
            output_text,
            ffmpeg_path_text,
            format_option,
            thread_number,
            sovrascrivi_checkbox,
            start_button
        ]
        .padding(Padding::new(10.0).left)
        .padding(Padding::new(10.0).right);

        return Container::new(col).center_x().into();
    }
}

fn main() {
    let _args: Vec<_> = env::args().collect();
    if _args.len() > 1 {
        println!("Running in headless mode");
        main_headless();
    } else {
        println!("Running in gui mode");
        let _ = Gui::run(Settings::default());
    }
}

fn main_gui(data: Gui) {
    // Controllo se esiste la cartella di input
    if !Path::new(data.input_folder.clone().as_str()).exists() {
        println!("La cartella di input non esiste");
        return;
    }

    // Creo la cartella di output nel caso non esiste, se esiste continua
    let output_folder_result = fs::create_dir_all(data.output_folder.clone());
    if output_folder_result.is_ok() || Path::new(data.output_folder.clone().as_str()).exists() {
        println!("Cartella output creata: {}", data.output_folder)
    } else {
        println!("Cartella output non creata: {}", data.output_folder);
        return;
    }

    // Creo una lista iteratore di stringhe con le canzoni
    let array_canzoni_temp = get_song_list(data.input_folder.clone());

    // Se non ci sono canzoni, termina il programma
    if array_canzoni_temp.is_empty() {
        println!("Non ci sono canzoni nella cartella di input.");
        return;
    } else {
        println!("Ci sono canzoni nella cartella di input.");
    }

    // Instanzio la struct
    // Nelle prossime versioni trasformerò tutto in una Lista
    let dati_condivisi: Canzoni = Canzoni {
        vettore_canzoni: array_canzoni_temp.clone(),
        posizione: 0,
        input_folder: data.input_folder,
        output_folder: data.output_folder,
        program: data.ffmpeg_path.to_string(),
        sovrascrivi: data.overwrite,
        formato: data.format_selected,
    };

    // Instanzio il lucchetto Mutex che usero per accedere ai dati condivisi
    let mutex_lock: Arc<Mutex<Canzoni>> = Arc::new(Mutex::new(dati_condivisi));

    // Inizio ciclo dei thread
    ciclo_threads(data.threads, mutex_lock, array_canzoni_temp.len());
}

fn main_headless() {
    // Instazio la variabile contenente gli argomenti
    let args = Args::parse();
    let threadcount: usize = args.threadcount;
    let input_folder_arg: String = args.input;
    let output_folder_arg: String = args.output;

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
    } else {
        println!("Ci sono canzoni nella cartella di input.");
    }

    // Controllo se l'estensione inserita è valtida
    let mut ext_is_valid: bool = false;
    for format in FORMATS {
        if args.formato.as_str() == format {
            ext_is_valid = true;
            break;
        }
    }
    if !ext_is_valid {
        println!("Formato non supportato");
        return;
    }

    // Instanzio la struct
    // Nelle prossime versioni trasformerò tutto in una Lista
    let dati_condivisi: Canzoni = Canzoni {
        vettore_canzoni: array_canzoni_temp.clone(),
        posizione: 0,
        input_folder: input_folder_arg,
        output_folder: output_folder_arg,
        program: args.program,
        sovrascrivi: args.sovrascrivi,
        formato: args.formato,
    };

    // Instanzio il lucchetto Mutex che usero per accedere ai dati condivisi
    let mutex_lock: Arc<Mutex<Canzoni>> = Arc::new(Mutex::new(dati_condivisi));

    // Inizio ciclo dei thread
    ciclo_threads(threadcount, mutex_lock, array_canzoni_temp.len());
}

fn ciclo_threads(
    threadcount: usize,
    mutex_lock: Arc<Mutex<Canzoni>>,
    numero_canzoni_totali: usize,
) {
    // Vettore mutabile per gestire i thread, specialmente la parte di join
    let mut thread_vector: Vec<thread::JoinHandle<()>> = vec![];
    for thread_id in 0..threadcount {
        // Necessario, se no non posso utilizzarlo (capisci perchè)
        let mutex_lock = mutex_lock.clone();

        println!("Thread '{}' ha iniziato", thread_id);

        // Qui metto in nuovo thread nell'array, è come una lista.
        thread_vector.push(thread::spawn(move || {
            // Ciclo che passa tutte le canzoni se necessario, quasi garantito che esca prima
            for _ in 0..numero_canzoni_totali {
                let result = thread_operation(&mutex_lock, numero_canzoni_totali, thread_id);
                if result == 1 {
                    break;
                }
            }
            println!("Thread '{}' ha finito", thread_id);
        }));
    }

    // In questo ciclo faccio il join dei thread, altrimenti il programma padre termina prima dei thread
    for thread_element in thread_vector {
        let _ = thread_element.join();
    }
}

// questa funzione contiene il codice eseguito dai thread
fn thread_operation(
    mutex_lock: &Arc<Mutex<Canzoni>>,
    numero_canzoni: usize,
    thread_id: usize,
) -> i32 {
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
    println!(
        "Thread '{}' ha iniziato `{}` {}/{}",
        thread_id,
        nome_canzone,
        posizione_temp + 1,
        numero_canzoni
    );

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
    let argomenti = [
        sovrascrivi_arg,
        "-loglevel",
        "panic",
        "-nostats",
        "-i",
        canzone_input_path.as_str(),
        "-vn",
        canzone_output_path.as_str(),
    ];

    // Istanzio ed eseguo il comando di ffmpeg con le impostazioni scelte
    let command = Command::new(program_temp)
        .args(argomenti)
        .spawn()
        .unwrap()
        .wait();
    // Verifico risultato del comando
    if command.is_err() {
        // metti questa print fuori dalla funzione
        println!("Errore nel thread {}, esco", thread_id);
        return -1;
    } else {
        println!(
            "Thread '{}' ha finito `{}` {}/{}",
            thread_id,
            nome_canzone,
            posizione_temp + 1,
            numero_canzoni
        );
        return 0;
    }
}

fn get_song_list(input_folder_arg: String) -> Vec<String> {
    // Creo una lista iteratore di stringhe con le canzoni
    let lista_canzoni = fs::read_dir(input_folder_arg.clone()).unwrap();

    let mut array_canzoni_temp: Vec<String> = vec![];
    // La converto in un array, altrimenti non riesco a passarla ai thread
    for elemento in lista_canzoni {
        array_canzoni_temp.push(elemento.unwrap().file_name().into_string().unwrap());
    }

    return array_canzoni_temp;
}
