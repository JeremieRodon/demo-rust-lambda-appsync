<script setup>
const open = defineModel();
</script>

<template>
  <Teleport to="body">
    <!-- Put this part before </body> tag -->
    <input type="checkbox" name="modal-toggle" class="modal-toggle" v-model="open" />
    <div class="modal z-40" role="dialog">
      <div class="modal-box">
        <slot></slot>
      </div>
    </div>
  </Teleport>
</template>
