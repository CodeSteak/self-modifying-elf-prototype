#define _GNU_SOURCE 1

#include <stdio.h>
#include <stdlib.h>

#include <errno.h>

#include <fcntl.h>

#include <unistd.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <sys/syscall.h>

#define BUFFER_LEN 4096

int main(int argc, char* argv[]) {

    if(argc <= 0) {
        printf("Need to be started with commandline arguments.\n");
        return 1;
    }

    int is_executable_in_path = 1;
    char* c = argv[0];
    while(*c != 0) {
        if(*c == '/') is_executable_in_path = 0;
        c++;
    }

    if (is_executable_in_path) {
        printf("This form of calling is currently unsupported.");
        return 1;
    }

    int memfd = syscall(SYS_memfd_create, "procstart", 0);
    if(memfd == -1) {
        perror("Unable to memfd_create");
        return 2;
    }

    int self_fd = open(argv[0], O_RDONLY);
    if(self_fd == -1) {
        perror("Unable to open this executable");
        return 3;
    }

    // Seek to payload
    int pos = lseek(self_fd, SELF_LEN, SEEK_SET);
    if(pos != SELF_LEN) {
        perror("Unable to lseek");
        return 4;
    }

    void* buffer = malloc(BUFFER_LEN);
    if((int)buffer == -1) {
        perror("Out of memory");
        return 5;
    }

    unsigned int payload_left = PAYLOAD_LEN;
    int last_read    = 0;
    do {
        int should_read = BUFFER_LEN < payload_left ? BUFFER_LEN : payload_left;
        last_read = read(self_fd, buffer, should_read);

        if (last_read < 0) {
            perror("Unable to read this executable");
            return 6;
        }else if (last_read > 0){
            int last_write = write(memfd, buffer, last_read);
            if(last_write != last_read) {
                perror("Unable to write all");
                return 6;
            }
        }
    }while(last_read != 0 && payload_left > 0);

    free(buffer);

    char extra_env[100];
    snprintf(extra_env, sizeof(extra_env), "SELF_OFFSET=%ld", (long)(PAYLOAD_LEN) + (long)(SELF_LEN));
    putenv(extra_env);

    fexecve(memfd, argv, environ);
    perror("Unable to execute.");
    
    printf("PAYLOAD_LEN=%ld SELF_LEN=%ld\n", (long)(PAYLOAD_LEN), (long)(SELF_LEN));
    return 42;
}
