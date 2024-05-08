pin do wyświetlacza:
- 1 mosi
- 2 sck
- 3 gnd
- 4 vcc
- 5 cs1
- 6 cs2
- 7 dc
- 8 rst

pin do przycisków:
- 1 mosi
- 2 sck
- 3 gnd
- 4 vcc
- 8 rst
- 9 cs
- 10 miso

można połączyć CS w 2 piny, ale to nadal daje 9, nie 8 które da radę popchać przez eth
CAP1188 wspiera 3 pinowy SPI, ale tutaj będzie problem z driverami?
można by też było użyć dekodera 2 to 4 line, użyć 2 pinów żeby zamienić je na CS1, CS2, CS3 oraz RESET, 
to z 10 zrobi 8 co będzie idealnie?
Przykładowe układy:

- SN74HC139D - dwukanałowy co nie potrzebujemy
- SN74LVC1G139DCTT - jednokanałowy mały, wygląda idealnie

SN74LVC1G139DCTT daje outputy typu low, może wymagać dodania NOTów np. 74AC04


https://github.com/collin80/SavvyCAN może się przydać do śledzenia co się dzieje na CAN
https://hackaday.com/2023/11/22/esp32-used-as-wireless-can-bus-reader/

okazuje się że CAP1188 ma reset na odwrót, trzeba odwrócić

Rozkodowanie Tourge Pro CSV:

* — multiply, example 256*A
/ — Divide, example A/10
+ — Add, example 256*A+B
– — Subtract, example B-10
& – Logical AND, example A&4
() — precedence operator, example (256*A+B)*0.1
{A:b} — returns the bth bit of the byte A. Least significant bit is 0, example A = 00001010b = 09h, {A:0} = 0; {A:1} = 1
[XX] – returns the value of a PID, where XX is the ID of the OBD2 sensor. i.e. [0d] (speed) or [ff1001] (gps speed)
Baro() returns barometer psia from [33] – Vehicle or [ff1270] – Phone.
A^B – returns A to the power of b
Log10(A) – returns the Log(base 10) of A
SIGNED(A) returns decimal value of A assuming highest bit is a sign bit
ABS(A) returns absolute value of A
VAL{sensor name} returns the value of the sensor, make sure this matches exactly. Sensor names are their untranslated names.

0_Niro_Cumulative Charge Current,CCC,2101,((ae<24)+(af<16)+(ag<8)+ah)/10,0,1000000,Ah,7E4

Hi!

R0 is the ‘raw’ variable (includes part of the response before the A,B,C start). The ABC variables start from the actual data after headers, lengths, ID bytes (etc, etc) are sent depending on the protocol used

But as far as the Nx variables go:

N0 is the same as A
N1 is the same as B
(etc)
for the variable naming thing – it’s basically a replacement for the A,B,C stuff – this is mainly due to the way the parser works. The A,B,C variables will continue to work and I have no plan to stop them being used, though if you get strange results, try with N0, N1, (etc) variables instead

End Quote.

FYI

D24V10F3 power step-down

Incoming pins:
1. +12V
2. GND
3. OBD CAN H
4. OBD CAN L
5. ING SIGNAL
6. CAN LISTEN H
7. CAN LISTEN L

crystal for can https://www.tme.eu/pl/details/3225-16m-sr/rezonatory-kwarcowe-smd/sr-passives/