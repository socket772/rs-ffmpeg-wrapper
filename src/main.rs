use num_cpus;
use std::sync::{Arc, Mutex};
use std::thread::{self};

fn main() {


	// Instanzio il lucchetto Mutex che userò per accedere al contatore
	// Devo usare Atomic reference counting, in questo caso è il migliore per gestire la concorrenza
	let mutex_lock = Arc::new(Mutex::new(0));

	// Devo impostare il valore iniziale del contatore, quindi prendo il contatore con lock
	let mut contatore = mutex_lock.lock().unwrap();
	// Imposto il valore del contatore
	*contatore = num_cpus::get();
	// Cancello la variabile contatore con drop. Questo è l'equivalente di lock, la funzione si limita a cancellare la variabile
	drop(contatore);

	println!("Questo computer ha {} thread", num_cpus::get());

	// Creo un vettore mutabile che conterrà tutte le informazioni di tutti i thread. Esso serve per effettuare il join, altrimenti il programma esce prima di aver finito tutti i thread
	let mut thread_vector:Vec<thread::JoinHandle<()>> = vec![];

	// In questo ciclo for vengono fatti partire i thread, non verranno eseguiti in ordine ovviamente
	for numero in 0..num_cpus::get() {

		// Creo una copia del lock
		let mutex_lock = Arc::clone(&mutex_lock);

		// Qui metto in nuovo thread nell'array, è come una lista.
		thread_vector.push(
			thread::spawn(move || {

				// Instanzio il contatore protetto dal mutex_lock e chiudo il lucchetto
				let mut contatore = mutex_lock.lock().unwrap();

				// Effettuo l'operazione voluta
				println!("Io sono il processo figlio {}, {}->{}", numero, *contatore, *contatore+1);
				*contatore = *contatore+1;
			}));
	}

	// In questo ciclo faccio il join dei thread, altrimenti il programma padre termina prima dei thread
	for thread_element in thread_vector {
		let _ = thread_element.join();
	}

	println!("Risultato finale = {}", *mutex_lock.lock().unwrap());
	println!("Risultato aspettato = {}", 2*num_cpus::get());
	

}


/*
fn esegui_comando(numero: usize) {

	let comando_string:String = format!("echo {}", numero);

	let comando = Command::new("sh")
	.arg("-c")
	.arg(comando_string)
	.output()
	.expect("Non sono riuscito ad eseguire il processo");	

	println!("{:?}", comando);
}
*/