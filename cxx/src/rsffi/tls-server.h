#pragma once
#include"core.h"

#ifdef __cplusplus
extern "C"{
#endif


TlsSerCon tls_config_single_cert_pem(FfiSlice certs, FfiSlice key, char* alpns); // alpns is a comma seperated string, can be null if invalid certs
TlsSniBui tls_config_sni_builder();                                              // null if invalid certs
TlsSniBui tls_config_sni_builder_with_pem(FfiSlice certs, FfiSlice key);         // null if invalid certs
bool tls_config_sni_add_pem(TlsSniBui builder, char* domain, FfiSlice certs, FfiSlice key);
TlsSerCon tls_config_sni_builder_build(TlsSniBui builder, char* alpns);
void tls_config_free(TlsSerCon config);
void tcp_upgrade_tls(FfiFuture fut, FfiStream stream, TlsSerCon conf);


#ifdef __cplusplus
}
#endif