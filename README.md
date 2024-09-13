# Projekt pri predmetu Programiranje 2


- avtorja: Lucija in Lev
- programski jezik: Rust

## Kako se projekt požene

Za zagon projekta je potrebno najprej zagnati Register (v terminalu na lokaciji datoteke z imenom Register napišemo `cargo run`), nato še Generator.

Nato za željena zaporedja pošljemo poizvedbe preko python programa.

### Sintaksa poizvedbe za zaporedje

```
body = {
    "range": {
        "from": from_number,
        "to": to_number,
        "step": step,
    },
    "parameters": list_of_parameters,
    "sequences": list_of_sequences,
}
```
Pri čemer je sintaksa zaporedij znotraj seznama:

```
{"name": name_of_sequence, 
 "parameters": list_of_parameters, 
 "sequences": list_of_sequences}
```

Primer: [^1]

```
body = {
    "range": {
        "from": 0,
        "to": 10,
        "step": 2,
    },
    "parameters": [1, 3],
    "sequences": [],
}
requests.post(url + "/Arithmetic", json=body)
```
[^1]: V tem primeru nam generator vrne [1.0, 7.0, 13.0, 19.0, 25.0, 31.0], torej člene od prvega, saj je "from" _0_, prvi člen smo v parametrih določili kot _1_, do enajstega (člen z indeksom "to" _10_), kjer jemljemo le vsakega drugega, saj je "step" _2_. Ker je to aritmetično zaporedje, velja a<sub>n</sub> = a<sub>(n-1)</sub> + _3_ (_3_ smo določili v parametrih).

## Delovanje

Naš projekt se najprej registrira na _127.0.0.1:7878_, kjer so vsi registrirani projekti. Potem posluša na našem naslovu _127.0.0.1:12345_. Če pride kakšna poizvedba (post request), jo prebere in ugotovi, kakšno zaporedje želi. 

Imamo želeno zaporedje 
: če imamo mi to zaporedje, ga generiramo in pošljemo poizvedovalcu (postamo na _127.0.0.1:12345/sequence/sequence_name_)

Nimamo
: če ga nimamo, pošljemo poizvedbo naključnemu drugemu projektu, ki ima želeno zaporedje med svojimi zaporedji. To počnemo tako, da vsakič po naključnem vrstnem redu pregledamo zaporedja projektov in izberemo prvega, ki ima zaporedje. Ko ga dobimo nazaj, ga pošljemo.

Nihče nima
: vrnemo prazno zaporedje, izpišemo napako "Nobody has {sequence_name}"

## Omejitve

Ta projekt ne deluje s poizvedbami za zaporedja, ki jih imamo, a potrebujejo podzaporedja, ki jih nimamo.
To je zato, ker so zaporedja s podzaporedji definirana na zaporedjih tipa `Sequence`, torej jih ne moremo narediti iz odziva drugih projektov, ki je json.

## Naša zaporedja

Zaporedja, ki jih ima naš projekt, so naslednja:

- Aritmetično:
    - parametra: začetni člen in korak
    - a<sub>n</sub> = a<sub>n-1</sub> + korak
- Geometrijsko:
    - parametra: začetni člen in faktor
    - a<sub>n</sub> = a<sub>n-1</sub> * faktor
- Konstantno:
    - parameter: začetni člen
    - a<sub>n</sub> = a<sub>0</sub>
- Sum:
    - parametra: dve zaporedji
    - a<sub>n</sub> = b<sub>n</sub> + c<sub>n</sub>
- Prod:
    - parametra: dve zaporedji
    - a<sub>n</sub> = b<sub>n</sub> * c<sub>n</sub>
- Drop:
    - parametra: zaporedje _b<sub>n</sub>_ in število izpuščenih členov _k_
    - a<sub>n</sub> = b<sub>n-k</sub> ; n >= k
- Linearna kombinacija:
    - parametri: trije skalarji _A_, _B_, _C_ in dve zaporedji _b<sub>n</sub>_, _c<sub>n</sub>_
    - a<sub>n</sub> = A * b<sub>n</sub> + B * c<sub>n</sub> + C
- Rekurzivno:
    - parametri: prva dva člena zaporedja _a<sub>0</sub>_, _a<sub>1</sub>_, faktorja _A_ in _B_
    - a<sub>n</sub> = A * a<sub>n-2</sub> + B * a<sub>n-1</sub>
- Povprečje:
    - parametri: dve zaporedji
    - a<sub>n</sub> = (b<sub>n</sub> + c<sub>n</sub>) / 2
- Ciklično:
    - parametri: zaporedje in dolžina cikla
    - a<sub>n</sub> = b<sub>n%k</sub>
- Alternirajoče:
    - parametri: zaporedje
    - a<sub>n</sub> = (-1)<sup>n</sup> * b<sub>n</sub>
- Zglajeno:
    - parameter: zaporedje
    - a<sub>n</sub> = (b<sub>n-1</sub> + b<sub>n</sub> + b<sub>n+1</sub>) / 3
