#include <err.h>
#include <stdio.h>
#include <sys/capability.h>
#include <sys/types.h>
#include <unistd.h>

int main(void)
{
    printf("Capabilities:\n");

    cap_t caps = cap_get_pid(getpid());

    for (cap_value_t c = 0; c < cap_max_bits(); ++c)
    {
        char *name = cap_to_name(c);
        cap_flag_value_t value;

        cap_get_flag(caps, c, CAP_EFFECTIVE, &value);

        if (value == CAP_SET)
        {
            printf("%s EFFECTIVE\n", name);
        }

        cap_get_flag(caps, c, CAP_PERMITTED, &value);

        if (value == CAP_SET)
        {
            printf("%s PERMITTED\n", name);
        }

        cap_get_flag(caps, c, CAP_INHERITABLE, &value);

        if (value == CAP_SET)
        {
            printf("%s INHERITABLE\n", name);
        }

        cap_free(name);
    }

    cap_free(caps);

    return 0;
}
