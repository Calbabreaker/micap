#ifndef I2C_CLEARBUSS_H
#define I2C_CLEARBUSS_H

/**
 * I2C_ClearBus
 * (http://www.forward.com.au/pfod/ArduinoProgramming/I2C_ClearBus/index.html)
 * (c)2014 Forward Computing and Control Pty. Ltd.
 * NSW Australia, www.forward.com.au
 * This code may be freely used for both private and commerical use
 */

#include <Wire.h>
/**
 * This routine turns off the I2C bus and clears it
 * on return SCL and SDA pins are tri-state inputs with PULLUPS
 * You need to call Wire.begin() after this to re-enable I2C
 * This routine does NOT use the Wire library at all.
 *
 * returns 0 if bus cleared
 *         1 if sclPin held low.
 *         2 if sdaPin held low by slave clock stretch for > 2sec
 *         3 if sdaPin held low after 20 clocks.
 */
int I2C_ClearBus();

/**
 * This routine turns off the I2C bus and clears it
 * on return scaPin and sclPin pins are tri-state inputs with PULLUPS
 * You need to call Wire.begin() after this to re-enable I2C
 * This routine does NOT use the Wire library at all.
 *
 * returns 0 if bus cleared
 *         1 if sclPin held low.
 *         2 if sdaPin held low by slave clock stretch for > 2sec
 *         3 if sdaPin held low after 20 clocks.
 */
int I2C_ClearBus(int sdaPin, int sclPin);

#endif
