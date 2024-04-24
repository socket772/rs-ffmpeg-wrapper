import sys
import subprocess

# Controllo se ci sono tutti gli argomenti necessari
if sys.argv.__len__() !=4:
    print("Servono 3 argomenti, cartella input, cartella output, numero thread e se mettere in pausa i test dopo la loro esecuzione")
    print("cartella cartella int")
    exit()

# Lista di estensioni da supportare
estensioni: list[str] = ["mp3", "m4a", "flac", "ogg", "wav", "aac", "m4b", "oga", "opus", "webm"]

# Estraggo gli argomenti
input: str = sys.argv[1]
output: str = sys.argv[2]
threads: str = sys.argv[3]

# Print di debug
# print("Estensioni: {} -> {}", estensioni, type(estensioni))
# print("Input: {} -> {}", input, type(input))
# print("Output: {} -> {}", output, type(output))
# print("Threads: {} -> {}", threads, type(threads))


# Test 1, conversione di tutte le estensioni
def test_1():
    for estensione in estensioni:
        print("Inizio test sull'estensione {estensione}, usando il numero di thread di default")
        subprocess.run(["cargo", "run", "-q", "--", "-i", input, "-o", output+"/debug_allformats_no_t", "-f", estensione])

        print("Inizio test sull'estensione {estensione}, usando il numero specificato")
        subprocess.run(["cargo", "run", "-q", "--", "-i", input, "-o", output+"/debug_allformats_t", "-f", estensione, "-t", threads])

    print("Test 1 finito")
    input("Premi invio per continuare")

# Test 2, conversione di tutte le estensioni ma i file sono presenti e non verranno sovrascritti
def test_2():
    for estensione in estensioni:
        print("Inizio test sull'estensione {estensione}, usando il numero di thread di default")
        subprocess.run(["cargo", "run", "-q", "--", "-i", input, "-o", output+"/debug_allformats_no_t", "-f", estensione])

        print("Inizio test sull'estensione {estensione}, usando il numero specificato")
        subprocess.run(["cargo", "run", "-q", "--", "-i", input, "-o", output+"/debug_allformats_t", "-f", estensione, "-t", threads])

    print("Test 2 finito")
    input("Premi invio per continuare")

# Test 3, conversione di tutte le estensioni ma i file sono presenti e verranno sovrascritti
def test_3():
    for estensione in estensioni:
        print("Inizio test sull'estensione {estensione}, usando il numero di thread di default")
        subprocess.run(["cargo", "run", "-q", "--", "-i", input, "-o", output+"/debug_allformats_no_t", "-f", estensione, "-s"])

        print("Inizio test sull'estensione {estensione}, usando il numero specificato")
        subprocess.run(["cargo", "run", "-q", "--", "-i", input, "-o", output+"/debug_allformats_t", "-f", estensione, "-t", threads, "-s"])

    print("Test 3 finito")
    input("Premi invio per continuare")

def main():
    print("Inizio dei test")
    test_1()
    test_2()
    test_3()

if __name__ == "__main__":
    main()
