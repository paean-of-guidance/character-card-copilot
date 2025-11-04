<script setup lang="ts">
import { ref, provide } from "vue";
import { RouterView } from "vue-router";
import Navbar from "@/components/Navbar.vue";
import Modal from "@/components/Modal.vue";
import NotificationToast from "@/components/NotificationToast.vue";
import type { Toast, ModalOptions } from "@/utils/notification";
import { generateId } from "@/utils/notification";
import { NotificationSymbol } from "@/composables/useNotification";
import { ModalSymbol } from "@/composables/useModal";

// Toast状态管理
const toasts = ref<Toast[]>([]);

// Modal状态管理
const modalOptions = ref<ModalOptions | null>(null);

// Toast方法
function showToast(
  type: Toast["type"],
  message: string,
  title?: string,
  duration?: number
) {
  const toast: Toast = {
    id: generateId(),
    type,
    title,
    message,
    duration,
    timestamp: Date.now()
  };

  toasts.value.push(toast);
}

function showSuccessToast(message: string, title?: string) {
  showToast("success", message, title || "操作成功");
}

function showWarningToast(message: string, title?: string) {
  showToast("warning", message, title || "警告");
}

function showInfoToast(message: string, title?: string) {
  showToast("info", message, title || "提示");
}

function showErrorToast(message: string, title?: string) {
  showToast("error", message, title || "错误");
}

// Modal方法
function showAlertModal(
  message: string,
  onConfirm?: () => void | Promise<void>,
  options?: Partial<ModalOptions>
): Promise<boolean> {
  return new Promise((resolve) => {
    modalOptions.value = {
      title: options?.title || "确认操作",
      message,
      type: options?.type || "warning",
      confirmText: options?.confirmText || "确认",
      cancelText: options?.cancelText || "取消",
      onConfirm: async () => {
        try {
          await onConfirm?.();
          resolve(true);
        } catch (error) {
          console.error("Modal confirm error:", error);
          resolve(false);
        }
      },
      onCancel: () => {
        options?.onCancel?.();
        resolve(false);
      }
    };
  });
}

// Modal事件处理
function handleModalConfirm() {
  modalOptions.value = null;
}

function handleModalCancel() {
  modalOptions.value = null;
}

function handleModalClose() {
  modalOptions.value = null;
}

// 提供全局方法
provide(NotificationSymbol, {
  showSuccessToast,
  showWarningToast,
  showInfoToast,
  showErrorToast
});

provide(ModalSymbol, {
  showAlertModal
});
</script>

<template>
    <div id="app" class="min-h-screen bg-gray-50">
        <Navbar />
        <main class="container mx-auto p-4">
            <RouterView />
        </main>

        <!-- 全局Modal组件 -->
        <Modal
            :options="modalOptions"
            @confirm="handleModalConfirm"
            @cancel="handleModalCancel"
            @close="handleModalClose"
        />

        <!-- 全局Toast通知组件 -->
        <NotificationToast :toasts="toasts" />
    </div>
</template>

<style scoped></style>
