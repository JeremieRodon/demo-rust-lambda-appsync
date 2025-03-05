<script setup>
import { computed, inject, ref, watch, watchEffect } from 'vue';

defineOptions({
  inheritAttrs: false,
});
const model = defineModel();
const error_model = defineModel('error');

const props = defineProps({
  disabled: { type: Boolean, default: false },
  class: { type: String, default: null },
  placeholder: { type: String, default: null },
  rules: {
    type: Array[Function],
    default: [],
  },
  value_formater: {
    type: Function,
    default: (v) => v,
  },
  display_formater: {
    type: Function,
    default: (v) => v,
  },
  edit_display_value: { type: Boolean, default: false },
  error_message_delay: {
    type: Number,
    default: 1000,
  },
  in_operation: { type: Boolean, default: false },
});

const local_model = ref(null);
function sync_with_model() {
  console.log('Update local_model from model');
  console.log(model.value);
  local_model.value = model.value;
}
sync_with_model();
watch(model, sync_with_model);

const textinput = ref(null);
const editing = ref(false);

const read_only = inject('read_only', false);

watchEffect(() => {
  if (editing.value) {
    console.log('Compute error_model');
    for (const rule of props.rules) {
      const rule_res = rule(local_model.value);
      if (rule_res != true) {
        error_model.value = rule_res;
        return;
      }
    }
    error_model.value = null;
  }
});

const local_model_invalid = computed(() => {
  console.log('Compute local_model_invalid');
  return error_model.value != null;
});

const timer_show_error = ref({ show: false, timer: null });
watch(local_model, () => {
  if (editing.value) {
    console.log('Update timer_show_error');
    if (local_model_invalid.value) {
      clearTimeout(timer_show_error.value.timer);
      if (props.error_message_delay > 0) {
        timer_show_error.value.show = false;
        timer_show_error.value.timer = setTimeout(() => {
          timer_show_error.value.show = true;
        }, props.error_message_delay);
      } else {
        timer_show_error.value.timer = null;
        timer_show_error.value.show = true;
      }
    } else {
      clearTimeout(timer_show_error.value.timer);
      timer_show_error.value.timer = null;
      timer_show_error.value.show = false;
    }
  }
});

const input_classes = computed(() => {
  if (editing.value) {
    if (timer_show_error.value.show) {
      return 'input-error';
    } else {
      return 'input-primary';
    }
  } else {
    return 'border-transparent';
  }
});

function get_carret_position(event) {
  const style = event.srcElement.currentStyle || window.getComputedStyle(event.srcElement);
  const text_width = parseInt(style.width);
  const cursor_offset = event.offsetX;
  const text_length = local_model.value ? local_model.value.length : 0;
  const caret_position = Math.max(0, Math.round((cursor_offset * text_length) / text_width));
  return caret_position;
}
function enter_edit_mode(caret_position) {
  if (!editing.value) {
    console.log('enter_edit_mode');
    editing.value = true;
    if (props.edit_display_value) {
      local_model.value = props.display_formater(model.value);
    }
    setTimeout(() => {
      textinput.value.focus();
      if (caret_position != null) {
        textinput.value.setSelectionRange(caret_position, caret_position);
      } else {
        const text_len = local_model.value ? local_model.value.length : 0;
        textinput.value.setSelectionRange(text_len, text_len);
      }
    }, 1);
  }
}

function exit_edit_mode() {
  if (editing.value) {
    console.log('exit_edit_mode');
    editing.value = false;
    if (local_model_invalid.value) {
      clearTimeout(timer_show_error.value.timer);
      timer_show_error.value.show = true;
      timer_show_error.value.timer = setTimeout(() => {
        timer_show_error.value.show = false;
      }, 5000);
    } else {
      console.log('Update model from value_formater(local_model)');
      model.value = props.value_formater(local_model.value);
    }
    sync_with_model();
  }
}
</script>

<template>
  <div class="size-fit" @click.stop.prevent="enter_edit_mode()">
    <slot name="label"></slot>
  </div>
  <div class="relative max-w-fit" :class="props.class">
    <div class="h-0 invisible">{{ placeholder }}</div>
    <div class="border border-transparent flex" :class="editing ? 'invisible' : null">
      <div
        class="content-center leading-snug text-nowrap overflow-x-auto"
        @click.stop.prevent="
          (event) => {
            if (!read_only && !disabled) {
              enter_edit_mode(get_carret_position(event));
            }
          }
        "
      >
        {{ display_formater(local_model) ? display_formater(local_model) : '&nbsp;' }}
      </div>
      <div
        v-show="!read_only && !disabled && !in_operation"
        class="cursor-pointer content-center tooltip tooltip-secondary before:text-xs before:font-light before:text-white hover:after:delay-500 hover:before:delay-500"
        data-tip="Modifier"
        @click.stop.prevent="enter_edit_mode()"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="mx-1 h-3/4 aspect-square fill-secondary"
          viewBox="0 0 24 24"
        >
          <path
            d="M19.71,8.04L17.37,10.37L13.62,6.62L15.96,4.29C16.35,3.9 17,3.9 17.37,4.29L19.71,6.63C20.1,7 20.1,7.65 19.71,8.04M3,17.25L13.06,7.18L16.81,10.93L6.75,21H3V17.25M16.62,5.04L15.08,6.58L17.42,8.92L18.96,7.38L16.62,5.04M15.36,11L13,8.64L4,17.66V20H6.34L15.36,11Z"
          />
        </svg>
      </div>
      <span v-show="in_operation" class="loading loading-spinner loading-sm text-primary"></span>
      <div class="grow"></div>
    </div>
    <div v-show="!read_only && editing" class="absolute top-0 left-0 w-[calc(100%+20px)] h-full">
      <input
        class="border rounded-lg leading-snug pl-2 -ml-2 w-full max-h-full placeholder:text-base-content/20"
        :class="input_classes"
        ref="textinput"
        :placeholder="placeholder"
        v-model.trim="local_model"
        v-bind="$attrs"
        :disabled="read_only || disabled"
        @keydown.enter="exit_edit_mode()"
        @focusout="exit_edit_mode()"
        @focusin="enter_edit_mode()"
      />
    </div>
    <div
      class="w-full absolute -left-2 pt-px text-xs font-normal text-error text-center text-nowrap overflow-visible max-h-4"
      :class="!timer_show_error.show ? 'invisible' : null"
    >
      {{ error_model ? error_model : '&nbsp;' }}
    </div>
  </div>
</template>
