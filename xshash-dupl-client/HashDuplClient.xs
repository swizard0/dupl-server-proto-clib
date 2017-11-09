
#include "EXTERN.h"
#include "perl.h"
#include "XSUB.h"

#include <include/dupl_client.h>

typedef dupl_client_t AIP__Kribrum__HashDuplClient;

MODULE = AIP::Kribrum::HashDuplClient  PACKAGE = AIP::Kribrum::HashDuplClient

PROTOTYPES: DISABLE

AIP::Kribrum::HashDuplClient
create(perl_class)
	char *perl_class;
	
	CODE:
		dupl_client_t dc = NULL;
		dupl_client_create(&dc);
		RETVAL = dc;
		
	OUTPUT:
		RETVAL


SV*
init(xshdupl_client, zmq_connect_addr, req_timeout_ms = 0)
	AIP::Kribrum::HashDuplClient xshdupl_client;
	SV *zmq_connect_addr;
	unsigned long req_timeout_ms;
	
	CODE:
		const char* zmq_connect_addr_c = (char *)SvPV_nolen(zmq_connect_addr);
		RETVAL = newSViv( dupl_client_init(xshdupl_client, zmq_connect_addr_c, req_timeout_ms) );
	
	OUTPUT:
		RETVAL

SV*
request(xshdupl_client, json)
	AIP::Kribrum::HashDuplClient xshdupl_client;
	SV* json;
	
	CODE:
		if(SvOK(json))
		{
			STRLEN len_s;
			const char* json_c = (char* )SvPV(json, len_s);
			
			if(json_c == NULL || len_s < 1) {
				RETVAL = newSVpv("\0", 0);
			}
			else
			{
				const char *rep_json = 0;
				size_t rep_json_length = 0;
				
				int status = dupl_client_request(xshdupl_client, json_c, len_s, &rep_json, &rep_json_length, 1);
				
				if(status == 0) {
					RETVAL = newSVpv(rep_json, rep_json_length);
				}
				else {
					RETVAL = newSVpv("\0", 0);
				}
			}
		}
		else {
			RETVAL = newSVpv("\0", 0);
		}
		
	OUTPUT:
		RETVAL

SV*
last_error_description(xshdupl_client)
	AIP::Kribrum::HashDuplClient xshdupl_client;
	
	CODE:
		RETVAL = newSVpv(dupl_client_last_error(xshdupl_client), 0);
		
	OUTPUT:
		RETVAL


void
DESTROY(xshdupl_client)
	AIP::Kribrum::HashDuplClient xshdupl_client;
	
	CODE:
		dupl_client_close(&xshdupl_client);



