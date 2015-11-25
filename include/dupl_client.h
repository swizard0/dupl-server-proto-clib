#ifndef _DUPL_CLIENT_H_
#define _DUPL_CLIENT_H_

#ifdef __cplusplus
extern "C"
{
#endif

    typedef struct dupl_client * dupl_client_t;
    int dupl_client_create( dupl_client_t *dc );
    int dupl_client_close( dupl_client_t *dc );

    int dupl_client_init(
        dupl_client_t dc,
        const char *zmq_connect_addr,
        unsigned long req_timeout_ms );

    int dupl_client_request(
        dupl_client_t dc,
        const char *req_json,
        size_t req_json_length,
        const char **rep_json,
        size_t *rep_json_length,
        int pretty_print );
    
    const char *dupl_client_last_error( dupl_client_t dc );

#ifdef __cplusplus
}
#endif

#endif /* _DUPL_CLIENT_H_ */
