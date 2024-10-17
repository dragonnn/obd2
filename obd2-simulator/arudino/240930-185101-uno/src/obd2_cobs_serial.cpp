#include <Arduino.h>

#include "mcp2515v2.h"
#include <PacketSerial.h>
#include <SPI.h>
#include <Wire.h>

// frame structures
struct can_frame canMsgInComing;
struct can_frame canMsgOutgoing;

// create objects/init board
MCP2515v2 mcp2515v2(10);

PacketSerial_<COBS, 0, 512> myPacketSerial;

void onPacketReceived(const uint8_t *buffer, size_t size) {
  canMsgOutgoing.can_id =
      buffer[0] | (buffer[1] << 8) | (buffer[2] << 16) | (buffer[3] << 24);
  canMsgOutgoing.can_dlc = buffer[4];
  memcpy(canMsgOutgoing.data, &buffer[5], canMsgOutgoing.can_dlc);
  mcp2515v2.sendMessage(MCP2515v2::TXB0, &canMsgOutgoing);
}

void setup() {
  myPacketSerial.begin(115200);
  myPacketSerial.setPacketHandler(&onPacketReceived);

  mcp2515v2.reset();
  mcp2515v2.enableOSM();
  mcp2515v2.setBitrate(CAN_500KBPS, MCP_8MHZ);
  mcp2515v2.setNormalMode();
  Serial.println("init end");
}

uint8_t outBuffer[512];
uint8_t inBuffer[512];

void loop() {
  myPacketSerial.update();
  if (mcp2515v2.readMessage(&canMsgInComing) == MCP2515v2::ERROR_OK) {
    uint32_t can_id = canMsgInComing.can_id;
    outBuffer[0] = can_id & 0xFF;
    outBuffer[1] = (can_id >> 8) & 0xFF;
    outBuffer[2] = (can_id >> 16) & 0xFF;
    outBuffer[3] = (can_id >> 24) & 0xFF;
    outBuffer[4] = canMsgInComing.can_dlc;
    memcpy(outBuffer + 5, canMsgInComing.data, canMsgInComing.can_dlc);
    myPacketSerial.send(outBuffer, canMsgInComing.can_dlc + 5);
  }
}
