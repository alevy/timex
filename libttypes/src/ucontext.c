#include <stdlib.h>
#include <ucontext.h>

ucontext_t *ucontext_alloc(void) {
  return (ucontext_t*)malloc(sizeof(ucontext_t));
}

ucontext_t *ucontext_new(void(*start)(int, char*), void* stack_pointer, int stack_size, ucontext_t* link, int argc, char* argv) {
  ucontext_t* ctx = (ucontext_t*)malloc(sizeof(ucontext_t));
  getcontext(ctx);
  ctx->uc_stack.ss_sp = stack_pointer;
  ctx->uc_stack.ss_size = stack_size;
  ctx->uc_link = link;
  makecontext(ctx, (void(*)(void))start, 4, link, ctx, argc, argv);
  return ctx;
}

void ucontext_free(ucontext_t *ctx) {
  free(ctx);
}
