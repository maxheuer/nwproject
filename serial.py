import serial
import time

def write_data(serial_port, data):
    flat_data = bytearray()
    for num in data:
        flat_data.extend(num.to_bytes(2, byteorder='little'))
    serial_port.write(flat_data)

def read_data(serial_port):
    """Read data from the serial port and print it as UTF-8."""
    if serial_port.in_waiting > 0:
        data = serial_port.read(serial_port.in_waiting)
        try:
            # Decode the raw bytes to UTF-8
            utf8_data = data.decode('utf-8')
            print(utf8_data, end='')  # Print raw UTF-8 data
        except UnicodeDecodeError:
            # Handle decoding errors gracefully
            print(data.decode('utf-8', errors='replace'), end='')

def rotate_first_entry(arr):
    """Shift the first entry in the array left by one bit."""
    arr[0] = (arr[0] << 1) & 0xFFFF  # Keep it within 16 bits
    return arr

if __name__ == "__main__":
    # Initialize 16-element array of 16-bit unsigned integers (first entry is 1, rest are zeroes)
    data = [1] + [0] * 15  # The first entry is 1, the rest are 0
    # data = [0xffff] * 16  # The first entry is 1, the rest are 0

    # Open serial port (update with correct port for your system)
    with serial.Serial('/dev/ttyACM0', baudrate=115200, timeout=None, parity=serial.PARITY_NONE, stopbits=serial.STOPBITS_ONE, bytesize=serial.EIGHTBITS) as ser:  # Update port for Linux or COM1 for Windows
        # write_data(ser, [0] * 16)
        while True:
            # Print any incoming data from the serial port as UTF-8
            read_data(ser)
            
            # Write the inverted and shifted data to the serial port
            # write_data(ser, data)
            
            # Shift the first entry left every 2 seconds
            data = rotate_first_entry(data)
            
            # Wait for 2 seconds before the next shift
            time.sleep(2)
