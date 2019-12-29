# unit-converter

<p>
This small program can convert between any two (or four) units! Except for the ones that haven't been added to the file or aren't compatible with each other :)

It can take inputs in several formats, including with abbreviations,
full unit names, unit names with typos (and plural suffixes), and complex units (like km/hour to ft/s). 
It comes with a large list of interesting 
unit conversions, but as long as you follow the correct format, a (virtually) unlimited number can be added.
The input file format should be as follows:
<br>
</p>

__Input Table Format:__
- case insensitive
- abbreviations in parentheses
- abbreviations optional
- abbreviations optional after first time unit is input
- abbreviations must be unique (note: Mm is the same as mm)
- spaces in unit names okay
- newlines to space out sections okay


__Examples:__

1 Kilometer (km) = 1000 Meters (m)
<br>
10 Decimeters = 1 Meter
<br>
1 Furlong = 220 Yards

<br>

The user input format is designed to be intuitive, but just in case, here are a few guidelines and examples: 
- use 'to', 'in', or '->' to indicate your desired conversion
- use '/' or 'per' between complex units (km/hour or kilometers per hr)
- put a space between the value and the first unit
- don't enter values that are too big for a 64 bit integer to hold :)

__Examples:__ 

2.4 meters in mm
<br>
42 furlongs/fortnight to mi/hour
<br>
540.00456 gigameters per month in attoparsecs/uc


<br>

__Note:__ Enter 'see units' at any time to see a full list of units available from the table.txt file

__Also note:__ If you don't have rustc installed, a .exe file is included in the target/release folder
(just make sure to include a copy of table.txt in that subfolder as well)

__Credits:__ the unit conversion table was a group effort, and my edit distance function was heavily based on a geeksforgeeks post
