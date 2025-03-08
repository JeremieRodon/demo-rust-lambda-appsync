<script setup>
import { computed, inject, ref, watch, watchEffect } from 'vue';

const model = defineModel();
const error_model = defineModel('error');
const props = defineProps({
  id: { type: String, default: null },
  name: { type: String, default: null },
  disabled: { type: Boolean, default: false },
  class: { type: String, default: null },
  rules: {
    type: Array[Function],
    default: [],
  },
  value_formater: {
    type: Function,
    default: (v) => v,
  },
});
defineOptions({
  inheritAttrs: false,
});

const input = ref();
function focus() {
  input.value.focus();
}
defineExpose({ focus });

const read_only = inject('read_only', false);
const show_error = ref({ show: false, force_show: false, timer: null });

const local_value_copy = ref(null);
function sync_with_model() {
  console.log('Update local_value_copy from model');
  console.log(model.value);
  local_value_copy.value = model.value;
}
sync_with_model();
watch(model, sync_with_model);

const error_message = computed(() => {
  if (props.disabled) {
    return null;
  }
  for (const rule of props.rules) {
    const rule_res = rule(local_value_copy.value);
    if (rule_res != true) {
      return rule_res;
    }
  }
  return null;
});
watchEffect(() => {
  if (error_message.value) {
    error_model.value = error_message.value;
  } else {
    error_model.value = null;
  }
});

const valid_input = computed(() => {
  return error_message.value == null;
});
watchEffect(() => {
  if (valid_input.value) {
    model.value = local_value_copy.value;
  }
});
watch(local_value_copy, () => {
  if (!local_value_copy.value || valid_input.value) {
    if (show_error.value.timer) {
      clearTimeout(show_error.value.timer);
      show_error.value.timer = null;
    }
    show_error.value.show = false;
    show_error.value.force_show = false;
  } else if (!show_error.value.force_show) {
    if (show_error.value.timer) {
      clearTimeout(show_error.value.timer);
      show_error.value.timer = null;
    }
    show_error.value.show = false;
    show_error.value.timer = setTimeout(() => {
      show_error.value.show = true;
    }, 1000);
  }
});

function immediate_feedback() {
  if (!valid_input.value) {
    if (show_error.value.timer) {
      clearTimeout(show_error.value.timer);
      show_error.value.timer = null;
    }
    show_error.value.show = true;
    show_error.value.force_show = true;
  } else {
    model.value = props.value_formater(local_value_copy.value);
  }
}
</script>

<template>
  <div class="flex flex-col" :class="props.class">
    <label class="label flex-col items-start">
      <div class="label-text font-bold"><slot></slot></div>
      <input
        v-model.trim="local_value_copy"
        :id="id"
        :name="name"
        ref="input"
        class="input input-bordered placeholder:text-base-content/20"
        :class="show_error.show ? ['input-error'] : []"
        v-bind="$attrs"
        :disabled="read_only || disabled"
        @keydown.enter="immediate_feedback"
        @focusout="immediate_feedback"
      />
    </label>
    <div
      class="w-full pt-px text-xs font-normal text-error text-center max-h-4"
      :class="!show_error.show ? 'invisible' : null"
    >
      {{ error_message ? error_message : '&nbsp;' }}
    </div>
  </div>
</template>
