import { create } from "zustand";

interface AddRemoteModelDrawerState {
  open: boolean;
  loading: boolean;
  providerId: string | null;
  providerType: string | null;
}

export const useAddRemoteModelDrawerStore = create<AddRemoteModelDrawerState>(
  () => ({
    open: false,
    loading: false,
    providerId: null,
    providerType: null
  })
);

// Modal actions
export const openAddRemoteModelDrawer = (
  providerId: string,
  providerType: string
) => {
  useAddRemoteModelDrawerStore.setState({
    open: true,
    providerId,
    providerType
  });
};

export const closeAddRemoteModelDrawer = () => {
  useAddRemoteModelDrawerStore.setState({
    open: false,
    loading: false,
    providerId: null,
    providerType: null
  });
};

export const setAddRemoteModelDrawerLoading = (loading: boolean) => {
  useAddRemoteModelDrawerStore.setState({
    loading
  });
};
