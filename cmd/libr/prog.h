#ifndef PROG_H
#define PROG_H

#include <stdint.h>
#include <stdlib.h>

typedef struct ffi_string {
  char *buf;
  int length;
  int capacity;
} ffi_string;

typedef struct ffi_string_array {
  ffi_string *arr;
  int length;
} ffi_string_array;

inline ffi_string ffi_string_array_index(ffi_string_array *array, int index) {
  return array->arr[index];
}

typedef struct compiler_host {
  ffi_string (*default_library_path)(void *);
  ffi_string (*get_current_directory)(void *);
  ffi_string (*new_line)(void *);
  ffi_string (*get_source_file_text)(void *, char *, char *, int32_t);

  void (*free_ffi_string)(ffi_string);
  void (*free_ffi_string_array)(ffi_string_array);

  void *data;
} compiler_host;

inline ffi_string call_default_library_path(compiler_host *host) {
  return (*host->default_library_path)(host->data);
}

inline ffi_string call_get_current_directory(compiler_host *host) {
  return (*host->get_current_directory)(host->data);
}

inline ffi_string call_new_line(compiler_host *host) {
  return (*host->new_line)(host->data);
}

inline ffi_string call_get_source_file_text(compiler_host *host, char *file_name, char *path, int32_t language_version ) {
  return (*host->get_source_file_text)(host->data, file_name, path, language_version);
}

inline void call_free_ffi_string(compiler_host *host, ffi_string s) {
  return (*host->free_ffi_string)(s);
}

inline void call_free_ffi_string_array(compiler_host *host, ffi_string_array a) {
  return (*host->free_ffi_string_array)(a);
}

typedef struct go_array {
  void *data;
  size_t len;
} go_array;

typedef struct program_options {
  ffi_string config_file_name;
  ffi_string_array root_files;
  compiler_host host;
} program_options;

typedef struct diagnostic {
  char *message;
  char *file_name;

} diagnostic;

#endif