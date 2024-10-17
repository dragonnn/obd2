#include "mcp2515v2.h"
#include <SPI.h>
#include <Wire.h>

// misc placeholders
#define log_all_codes false
#define PID_RPM 0x0C
#define PID_OutsideTemp 0x46
#define PID_FuelTank 0x2F
#define PID_EngineLoad 0x04
#define PID_CoolantTemp 0x05
#define SCREEN_WIDTH 128
#define SCREEN_HEIGHT 32
#define OLED_RESET 4    // not sure what this is hahahaha
#define init_delay 3000 // duration that the startup screen is displayed.

// vanity placeholders
#define DEVICE_SERIAL_NUMBER "G0101"
#define DEVICE_NAME "OBD Scan Gauge Device"
#define HARDWARE_VERSION "Hw2.0"
#define DATE_OF_MFG "11/01/2020"
#define FIRMWARE_VERSION "2.0"
#define FIRMWARE_BUILD "01"

// frame structures
struct can_frame canMsg;
struct can_frame canMsgOutgoing;

// create objects/init board
MCP2515v2 mcp2515v2(10);

// timer vars
// max value of an unsigned long is ~2^32...this arduino will run for 50 days
// and then who knows what will happen when the millis() exceeds the max!
unsigned long last_rpm = 0;
unsigned long last_temp = 0;
bool last_read = false;

// display coordinate calculation vars
int16_t xs1, ys1;
uint16_t w, h;

// data vars
int rpm = 9999;
int eng_temp = 999;
int consecutive_ms_button_pressed = 0;
int incoming_byte_serial[11];

String BuildMessage = "";

// function to request OBD data
void requestDataOBD(unsigned long int pid) {
  canMsgOutgoing.can_id = 0x7DF; // request
  canMsgOutgoing.can_dlc = 8;    // length of data frame
  canMsgOutgoing.data[0] = 0x02; // ?
  canMsgOutgoing.data[1] = 0x01; // ?
  canMsgOutgoing.data[2] = pid;  // OBD PID that we are requesting
  canMsgOutgoing.data[3] = 0x00; // zeros
  canMsgOutgoing.data[4] = 0x00;
  canMsgOutgoing.data[5] = 0x00;
  canMsgOutgoing.data[6] = 0x00;
  canMsgOutgoing.data[7] = 0x00;
  Serial.println("sending can msg\n");
  for (uint8_t i = 0; i <= 7; i++) {
    Serial.print(canMsgOutgoing.data[i] < 16 ? "0" : "");
    Serial.print(canMsgOutgoing.data[i], HEX);
    Serial.print(" ");
  }
  mcp2515v2.sendMessage(&canMsgOutgoing);
  Serial.println("end sending");
}

// 014 0: 49 02 01 FF FF FF 1: FF FF FF FF FF FF FF 2: FF FF FF FF FF FF FF
void responseDataOBD(unsigned char data[], int size) {
  // return;
  canMsgOutgoing.can_id = 0x7E8; // response
  canMsgOutgoing.can_dlc = size; // length of data frame
  for (uint8_t i = 0; i <= size; i++) {
    canMsgOutgoing.data[i] = data[i];
  }
  /*Serial.println("sending can msg\n");
  for (uint8_t i = 0; i <= 7; i++) {
    Serial.print(canMsgOutgoing.data[i] < 16 ? "0" : "");
    Serial.print(canMsgOutgoing.data[i], HEX);
    Serial.print(" ");
  }
  Serial.println("end sending");*/
  BuildMessage = "";
  Serial.print("T: ");
  Serial.print(canMsgOutgoing.can_id);
  Serial.print(",");
  for (int i = 0; i < canMsgOutgoing.can_dlc; i++) {
    BuildMessage = BuildMessage + canMsgOutgoing.data[i] + ",";
  }
  Serial.println(BuildMessage);
  mcp2515v2.sendMessage(&canMsgOutgoing);
}

void setup() {
  Serial.begin(115200);
  Serial.println("init");

  // init can board
  mcp2515v2.reset();
  mcp2515v2.enableOSM();
  mcp2515v2.setBitrate(CAN_500KBPS,
                       MCP_8MHZ); // Your vehicle may use a different speed!
  mcp2515v2.setNormalMode();
  Serial.println("init end");
}

void loop() {
  char rndCoolantTemp = random(22, 38);
  char rndRPM = random(1, 55);
  char rndSpeed = random(0, 255);
  char rndIAT = random(0, 255);
  char rndMAF = random(0, 255);
  char rndAmbientAirTemp = random(0, 200);
  char rndCAT1Temp = random(1, 55);

  // GENERAL ROUTINE
  unsigned char SupportedPID[8] = {1, 2, 3, 4, 5, 6, 7, 8};
  unsigned char MilCleared[7] = {4, 65, 63, 34, 224, 185, 147};

  // SENSORS
  unsigned char CoolantTemp[7] = {0x03, 0x41, 0x05, rndCoolantTemp,
                                  0xaa, 0xaa, 0xaa};
  unsigned char rpm[7] = {0x03, 0x41, 12, rndRPM, 0xaa, 0xaa, 0xaa};
  unsigned char vspeed[7] = {0x03, 0x41, 13, rndSpeed, 0xaa, 0xaa, 0xaa};
  unsigned char IATSensor[7] = {0x03, 0x41, 15, rndIAT, 0, 185, 147};
  unsigned char MAFSensor[7] = {0x03, 0x41, 16, rndMAF, 0, 185, 147};
  unsigned char AmbientAirTemp[7] = {0x03, 0x41, 70, rndAmbientAirTemp,
                                     0,    185,  147};
  unsigned char CAT1Temp[7] = {0x03, 0x41, 60, rndCAT1Temp, 224, 185, 147};
  unsigned char CAT2Temp[7] = {0x03, 0x41, 61, rndCAT1Temp, 224, 185, 147};
  unsigned char CAT3Temp[7] = {0x03, 0x41, 62, rndCAT1Temp, 224, 185, 147};
  unsigned char CAT4Temp[7] = {0x03, 0x41, 63, rndCAT1Temp, 224, 185, 147};
  unsigned char FuelRate[7] = {0x03, 0x41, 148, rndCAT1Temp, 224, 185, 147};
  unsigned char Speed[7] = {0x03, 0x41, 19, rndCAT1Temp, 224, 185, 147};
  unsigned char VIN_1[8] = {0x10, 0x14, 0x49, 0x02, 0x01, 0x55, 0x35, 0x59};
  unsigned char VIN_2[8] = {0x30, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00};
  unsigned char VIN_3[8] = {0x21, 0x48, 0x35, 0x46, 0x31, 0x41, 0x47, 0x4e};
  unsigned char VIN_4[8] = {0x22, 0x4c, 0x30, 0x32, 0x39, 0x35, 0x32, 0x33};
  // Data:                                             61    01    ff    ff ff
  // ff
  unsigned char BMS_1[8] = {0x10, 0x3d, 0x61, 0x01,
                            0xFF, 0xFF, 0xFF, 0xFF}; // FIRST FRAME
  unsigned char BMS_2[8] = {0x30, 0x00, 0x00, 0x00,
                            0x00, 0x00, 0x00, 0x00}; // FLOW CONTROL
  // Data:                                       be    11   a0     16    f8 80
  // 00
  unsigned char BMS_3[8] = {0x21, 0xbe, 0x11, 0xa0,
                            0x16, 0xf8, 0x80, 0x00}; // CON FRAME SEQ 1
  // Data:                                       00    0f    7e    20    1c 1c
  // 1c
  unsigned char BMS_4[8] = {0x22, 0x00, 0x0f, 0x7e,
                            0x20, 0x1c, 0x1c, 0x1c}; // CON FRAME SEQ 2
  // Data:                                       1c    1c    1e    00    22 ce
  // 02
  unsigned char BMS_5[8] = {0x23, 0x1c, 0x1c, 0x1e,
                            0x00, 0x22, 0xce, 0x02}; // CON FRAME SEQ 3
  // Data:                                       ce    31    00    ff    77 00
  // 01
  unsigned char BMS_6[8] = {0x24, 0xce, 0x31, 0x00,
                            0xff, 0x77, 0x00, 0x01}; // CON FRAME SEQ 4
  // Data:                                       88    d4    00    01    88 7b
  // 00
  unsigned char BMS_7[8] = {0x25, 0x88, 0xd4, 0x00,
                            0x01, 0x88, 0x7b, 0x00}; // CON FRAME SEQ 5
  // Data:                                       00    8d    5f    00    00 89
  // 68
  unsigned char BMS_8[8] = {0x26, 0x00, 0x8d, 0x5f,
                            0x00, 0x00, 0x89, 0x68}; // CON FRAME SEQ 6
  // Data:                                       00    9d    0c    af    49 00
  // 01
  unsigned char BMS_9[8] = {0x27, 0x00, 0x9d, 0x0c,
                            0xaf, 0x49, 0x00, 0x01}; // CON FRAME SEQ 7
  // Data:                                       00    00    00    00    03 e8
  // 00
  unsigned char BMS_10[8] = {0x28, 0x00, 0x00, 0x00,
                             0x00, 0x03, 0xe8, 0x00}; // CON FRAME SEQ 7

  if (mcp2515v2.readMessage(&canMsg) == MCP2515v2::ERROR_OK) {
    /*Serial.println("readed can msg");
    Serial.println(canMsg.can_id, HEX);
    Serial.println("pid");
    Serial.println(canMsg.data[2], HEX);

    if (canMsg.can_id == 0x7eA) { // response from ecu

      if (canMsg.data[2] == PID_CoolantTemp) { // Coolant

        //eng_temp = (9.0*(canMsg.data[3] - 40)/5.0+32.0); // there are formulas
    for turning the data bytes Serial.println("got eng_temp");
      }
      if (canMsg.data[2] == PID_RPM) { // ICE RPM
        //rpm = (canMsg.data[3]*256 + canMsg.data[4])/4; // same here. Some
    codes use more than one byte to store
                                                        // the svalue. The real
    RPM is a conjugate of two
                                                        // bytes, [3] and [4].

        Serial.println("got rpm");
      }

    }*/
    Serial.print("R: ");
    Serial.print(canMsg.can_id);
    Serial.print(",");

    for (int i = 0; i < canMsg.can_dlc; i++) {
      BuildMessage = BuildMessage + canMsg.data[i] + ",";
    }
    Serial.println(BuildMessage);

    // Check wich message was received.
    if (BuildMessage == "2,1,0,0,0,0,0,0,") {
      responseDataOBD(SupportedPID, 8);
    }
    if (BuildMessage == "2,1,1,0,0,0,0,0,") {
      responseDataOBD(MilCleared, 7);
    }

    // SEND SENSOR STATUSES
    if (BuildMessage == "2,1,5,0,0,0,0,0,") {
      responseDataOBD(CoolantTemp, 7);
    }
    if (BuildMessage == "2,1,12,0,0,0,0,0,") {
      responseDataOBD(rpm, 7);
    }
    if (BuildMessage == "2,1,13,0,0,0,0,0,") {
      responseDataOBD(vspeed, 7);
    }
    if (BuildMessage == "2,1,15,0,0,0,0,0,") {
      responseDataOBD(IATSensor, 7);
    }
    if (BuildMessage == "2,1,16,0,0,0,0,0,") {
      responseDataOBD(MAFSensor, 7);
    }
    if (BuildMessage == "2,1,70,0,0,0,0,0,") {
      responseDataOBD(AmbientAirTemp, 7);
    }
    if (BuildMessage == "2,1,60,0,0,0,0,0,") {
      responseDataOBD(CAT1Temp, 7);
    }
    if (BuildMessage == "2,1,61,0,0,0,0,0,") {
      responseDataOBD(CAT2Temp, 7);
    }
    if (BuildMessage == "2,1,62,0,0,0,0,0,") {
      responseDataOBD(CAT3Temp, 7);
    }
    if (BuildMessage == "2,1,63,0,0,0,0,0,") {
      responseDataOBD(CAT4Temp, 7);
    }
    if (BuildMessage == "2,1,148,0,0,0,0,0,") {
      responseDataOBD(FuelRate, 7);
    }
    if (BuildMessage == "2,1,19,0,0,0,0,0,") {
      responseDataOBD(Speed, 7);
    }
    if (BuildMessage == "2,9,2,0,0,0,0,0,") {
      Serial.println("VIN REQUEST");
      responseDataOBD(VIN_1, 8);
      delay(10);
      responseDataOBD(VIN_2, 8);
      delay(10);
      responseDataOBD(VIN_3, 8);
      delay(10);
      responseDataOBD(VIN_4, 8);
    }
    if (BuildMessage == "2,33,1,0,0,0,0,0,") {
      Serial.println("VIN REQUEST");
      responseDataOBD(BMS_1, 8);
      delay(10);
      responseDataOBD(BMS_2, 8);
      delay(10);
      responseDataOBD(BMS_3, 8);
      delay(10);
      responseDataOBD(BMS_4, 8);
      delay(10);
      responseDataOBD(BMS_5, 8);
      delay(10);
      responseDataOBD(BMS_6, 8);
      delay(10);
      responseDataOBD(BMS_7, 8);
      delay(10);
      responseDataOBD(BMS_8, 8);
      delay(10);
      responseDataOBD(BMS_9, 8);
      delay(10);
      responseDataOBD(BMS_10, 8);
    }
    BuildMessage = "";
    Serial.println("");
    Serial.println("");
  }

  /*  // "queue" data update requests
  if ((millis() - last_rpm) > 1000) { // change these values here to change the
  update frequency if (last_read) { requestDataOBD(PID_RPM); last_read = false;
    } else {
      requestDataOBD(PID_CoolantTemp);
      last_read = true;
    }
    // update counter
    last_rpm = millis();
  }*/
}
