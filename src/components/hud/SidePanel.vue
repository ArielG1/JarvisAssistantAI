<script setup lang="ts">
defineProps<{
  side: "left" | "right"
  visible: boolean
}>()
</script>

<template>
  <transition :name="side === 'left' ? 'panel-left' : 'panel-right'">
    <aside
      v-show="visible"
      :class="[
        'fixed top-12 bottom-16 z-20 w-64',
        'bg-jarvis-panel/80 backdrop-blur-md',
        'border border-jarvis-border/50',
        'overflow-y-auto',
        side === 'left' ? 'left-0 rounded-r-lg' : 'right-0 rounded-l-lg',
      ]"
    >
      <div class="p-4 h-full flex flex-col">
        <slot />
      </div>
    </aside>
  </transition>
</template>

<style scoped>
.panel-left-enter-active,
.panel-left-leave-active {
  transition: transform 0.3s ease, opacity 0.3s ease;
}
.panel-left-enter-from,
.panel-left-leave-to {
  transform: translateX(-100%);
  opacity: 0;
}

.panel-right-enter-active,
.panel-right-leave-active {
  transition: transform 0.3s ease, opacity 0.3s ease;
}
.panel-right-enter-from,
.panel-right-leave-to {
  transform: translateX(100%);
  opacity: 0;
}
</style>
