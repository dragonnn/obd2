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