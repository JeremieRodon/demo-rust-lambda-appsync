import { fetchAuthSession } from 'aws-amplify/auth';
import { computed, reactive } from 'vue';

export async function getUserInfos() {
  const {
    idToken: { payload },
  } = (await fetchAuthSession()).tokens ?? {};
  const groups = payload['cognito:groups'] ?? [];
  const is_admin = groups.indexOf('Admins') >= 0;
  const user = {
    id: payload.sub,
    email: payload.email,
    groups,
    is_admin,
  };
  return user;
}

const _alerts = reactive(new Map());
export const alerts = computed(() => {
  return [..._alerts.values()];
});

import { v4 as uuidv4 } from 'uuid';
export function alert(title, alert_class, message, timeout) {
  const id = uuidv4();
  const timer_handler = setTimeout(() => {
    _alerts.delete(id);
  }, timeout);
  const error_wrapper = {
    title,
    alert_class,
    message,
    timer_handler,
    close: () => {
      clearTimeout(timer_handler);
      _alerts.delete(id);
    },
  };
  _alerts.set(id, error_wrapper);
}
export function alert_error(message, timeout = 5000) {
  alert('Error', 'alert-error', message, timeout);
}
export function alert_success(message, timeout = 5000) {
  alert('Success', 'alert-success', message, timeout);
}
export function alert_warning(message, timeout = 5000) {
  alert('Warning', 'alert-warning', message, timeout);
}
export function alert_info(message, timeout = 5000) {
  alert('Info', 'alert-info', message, timeout);
}

export function alert_appsync_error(appsync_error_response, message, timeout = 5000) {
  console.error(appsync_error_response);
  for (const error of appsync_error_response.errors) {
    let error_type;
    if (error.errorType) {
      if (error.errorType.startsWith('Lambda:')) {
        error_type = `[${error.errorType.substring(7)}] `;
      } else {
        error_type = `[${error.errorType}] `;
      }
    } else {
      error_type = '';
    }
    const error_message = error.message;
    alert_error(`${message} (${error_type}${error_message})`, timeout);
  }
}

export function team_to_displayname(team) {
  if (team == 'RUST') {
    return 'Team Rust';
  } else if (team == 'PYTHON') {
    return 'Team Python';
  } else if (team == 'JS') {
    return 'Team Javascript';
  } else if (team == 'VTL') {
    return 'Team Velocity';
  } else {
    return 'No Team';
  }
}
