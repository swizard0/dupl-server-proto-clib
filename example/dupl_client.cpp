#define _WITH_GETLINE
#include <stdio.h>
#include <string.h>
#include <dupl_client.h>

const unsigned long default_request_timeout_ms = 3000;

int main(int argc, char **argv)
{
    if ( argc < 2 )
    {
        fprintf( stderr, "Usage: %s [--pretty-print] <zmq connect address>\n", argv[ 0 ] );
        return 1;
    }

    int pretty_print = 0;
    const char *zmq_connect_addr = NULL;
    if ( ( argc > 2 ) && ( strcmp( argv[ 1 ], "--pretty-print" ) == 0 ) )
    {
        pretty_print = 1;
        zmq_connect_addr = argv[ 2 ];
    }
    else
        zmq_connect_addr = argv[ 1 ];
    
    
    struct Client
    {
        dupl_client_t client;

        Client() : client( 0 ) { }
        ~Client()
        {
            if ( client != 0 )
                dupl_client_close( &client );
        }
    } c;

    if ( dupl_client_create( &c.client ) != 0 )
    {
        fprintf( stderr, "dupl_client_create failed (probably out of memory)\n" );
        return 1;
    }

    if ( dupl_client_init( c.client, zmq_connect_addr, default_request_timeout_ms ) != 0 )
    {
        fprintf( stderr, "dupl_client_init failed: %s\n", dupl_client_last_error( c.client ) );
        return 1;
    }

    char *line = 0;
    size_t line_length = 0;

    for ( ;; )
    {
        ssize_t bytes_read = getline( &line, &line_length, stdin );
        if ( bytes_read <= 0 )
            break;

        size_t json_length = static_cast<size_t>( bytes_read );
        const char *json = line;

        if ( json[ json_length - 1 ] == '\n' )
            json_length--;

        const char *reply = 0;
        size_t reply_length = 0;
        int ret = dupl_client_request( c.client, json, json_length, &reply, &reply_length, pretty_print );
        if ( ret > 0 )
        {
            fprintf( stderr, "dupl_client_request failed: %s\n", dupl_client_last_error( c.client ) );
            return 1;
        }
        else if ( ret < 0 )
            printf("timed out\n");
        else
            printf("%.*s\n", static_cast<int>(reply_length), reply);
        fflush(stdout);
    }    
    
    return 0;
}
