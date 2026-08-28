<script setup lang="ts">
import { ref } from "vue";
import BootSequence from "./views/BootSequence.vue";
import HUD from "./views/HUD.vue";

const bootDone = ref(false);
const bootSuccess = ref(false);

function onBootComplete(success: boolean) {
  bootDone.value = true;
  bootSuccess.value = success;
}
</script>

<template>
  <BootSequence v-if="!bootDone" @complete="onBootComplete" />
  <Transition name="fade" appear>
    <HUD v-if="bootDone && bootSuccess" />
  </Transition>
</template>

<style>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.8s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
