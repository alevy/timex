#include <stdlib.h>
#include <ucontext.h>

ucontext_t *ucontext_alloc(void) {
  return (ucontext_t*)malloc(sizeof(ucontext_t));
}

ucontext_t *ucontext_new(void(*trampoline)(void*), void* stack_pointer, int stack_size, void* ud, ucontext_t* link) {
  ucontext_t* ctx = (ucontext_t*)malloc(sizeof(ucontext_t));
  getcontext(ctx);
  ctx->uc_stack.ss_sp = stack_pointer;
  ctx->uc_stack.ss_size = stack_size;
  ctx->uc_link = link;
  makecontext(ctx, (void(*)(void))trampoline, 1, ud);
  return ctx;
}

void ucontext_free(ucontext_t *ctx) {
  free(ctx);
}
