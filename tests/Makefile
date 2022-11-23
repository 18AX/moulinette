CC?=gcc
CFLAGS=-Werror -Wextra -Wall -pedantic -std=c99
LFLAGS= -static

OBJECT=main.o
TARGET=test

all: $(TARGET)

$(TARGET): $(OBJECT)
	$(CC) $(LFLAGS) $(OBJECT) -o $(TARGET)

run: $(TARGET)
	./$(TARGET)

clean:
	$(RM) $(OBJECT) $(TARGET)