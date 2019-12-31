# unit-converter

<p>
This program can convert between any two (or four) units! The program can currently handle conversions such as mi/hr -> m/s, but not things like Watts -> J/s (yet). The program should be able to convert between any compatible units that have been added to the table.txt file. 

Conversions can be input in several formats, including with abbreviations,
full unit names, unit names with typos (and plural suffixes), and complex units (like km/hour to ft/s). 
It comes with a large list of interesting 
unit conversions, but as long as you follow the correct format, a (virtually) unlimited number can be added.
The input file format should be as follows:
<br>
</p>

__Input Table Format:__
- Case insensitive
- Abbreviations in parentheses
- Abbreviations optional
- Abbreviations optional after first time unit is input
- Abbreviations must be unique (note: Mm is the same as mm)
- Spaces in unit names okay
- Newlines to space out sections okay


__Examples:__

1 Kilometer (km) = 1000 Meters (m)
<br>
10 Decimeters = 1 Meter
<br>
1 Furlong = 220 Yards

<br>

The user input format is designed to be intuitive, but just in case, here are a few guidelines and examples: 
- Use 'to', 'in', or '->' to indicate your desired conversion
- Use '/' or 'per' between complex units (km/hour or kilometers per hr)
- Put a space between the value and the first unit
- Don't enter values that are too big for a 64 bit integer to hold :)

__Examples:__ 

2.4 meters in mm
<br>
42 furlongs/fortnight to mi per hour
<br>
540.00456 Angstroms per month -> attoparsecs/uc

<br>

__How it works:__
- Builds a directed action graph from input table (uses petgraph crate)
  - Edge weights represent conversion factors
  - Nodes are Unit structs, which currently store a full name and abbreviation
  - Reverse edges with weights of 1/(factor of original edge) are also added
- Unit lookups are done using an allowed edit distance to account for differences such as 'feet' vs. 'foot' or 'inches' vs 'inch'

<br>

__Limitations:__
- Cannot currently perform conversions between units that are combinations of other units (Watts -> J/s)
- Crashes on invalid input (although with edit distance allowance this is rare)

<br>

__Other Notes:__ 
- Enter 'see units' at any time to see a full list of units available from the table.txt file
- If you don't have rustc installed, a .exe file is included in the top level folder (make sure to download table.txt as well as the .exe)
- If you find any bugs or add units to the conversion table, please let me know or submit a pull request!

<br>

__Credits:__ The unit conversion table (table.txt) was a group effort, and my dynamic programming edit distance function was heavily based on a geeksforgeeks post (https://www.geeksforgeeks.org/edit-distance-dp-5/)
