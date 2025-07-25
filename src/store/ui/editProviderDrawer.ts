import { create } from "zustand";

interface EditProviderDrawerState {
  open: boolean;
  loading: boolean;
  providerId: string | null;
}

export const useEditProviderDrawerStore = create<EditProviderDrawerState>(
  () => ({
    open: false,
    loading: false,
    providerId: null,
  }),
);

// Modal actions
export const openEditProviderDrawer = (providerId: string) => {
  useEditProviderDrawerStore.setState({
    open: true,
    providerId,
  });
};

export const closeEditProviderDrawer = () => {
  useEditProviderDrawerStore.setState({
    open: false,
    loading: false,
    providerId: null,
  });
};

export const setEditProviderDrawerLoading = (loading: boolean) => {
  useEditProviderDrawerStore.setState({
    loading,
  });
};
