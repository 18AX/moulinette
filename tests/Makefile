CC?=gcc
CFLAGS=-Werror -Wextra -Wall -pedantic -std=c99
LFLAGS= -lcap -static

OBJECT=main.o
TARGET=test

all: $(TARGET)

$(TARGET): $(OBJECT)
	$(CC) $(OBJECT) -o $(TARGET) $(LFLAGS) 

run: $(TARGET)
	./$(TARGET)

clean:
	$(RM) $(OBJECT) $(TARGET)