#!/bin/sh

set -xe
cd lists

# Random dictionary lists
curl --location https://github.com/dwyl/english-words/raw/master/words.txt > dwyl.txt
curl --location https://archive.org/download/mobywordlists03201gut/SINGLE.TXT > single.txt
curl --location https://archive.org/download/mobywordlists03201gut/ACRONYMS.TXT > acronyms.txt
curl --location https://archive.org/download/mobywordlists03201gut/COMPOUND.TXT > compound.txt
curl --location https://archive.org/download/mobywordlists03201gut/NAMES.TXT > names.txt
# RockYou password list
curl --location https://github.com/brannondorsey/naive-hashcat/releases/download/data/rockyou.txt > rockyou.txt

# EFF's diceware dictionary
curl --location https://www.eff.org/files/2016/07/18/eff_large_wordlist.txt | sed 's/^[1-6]* //' > diceware.txt

